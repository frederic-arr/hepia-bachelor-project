use std::fs::Permissions;
use std::io::{BufReader, Read as _, Write as _};
use std::os::fd::{AsFd as _, OwnedFd};
use std::os::unix::fs::PermissionsExt as _;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context as _, Result, anyhow, bail};
use cos_proto_reconciler::{
    Phase,
    Resource,
    ResourceResponse,
    Status,
    ValidateResponse,
};
use rustix::fs::{
    AtFlags,
    CWD,
    Gid,
    Mode,
    OFlags,
    ResolveFlags,
    StatxFlags,
    StatxTimestamp,
    fsync,
    linkat,
    openat2,
    statx,
    unlinkat,
};
use rustix::io::Errno;
use rustix::process::{getgid, getuid};
use serde::{Deserialize, Serialize};
use sha3::Digest as _;

use crate::resources::{FilePermissions, FileState, FileType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticFileReconciler {
    pub root: PathBuf,
}

pub type StaticFileResource =
    Resource<StaticFileSpec, StaticFileDerivedSpec, FileState>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticFileSpec {
    pub path: PathBuf,
    pub content: String,
    pub owner_gid: Option<u32>,
    pub readable_by_group: bool,
    pub readable_by_others: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticFileDerivedSpec {
    pub size: u64,
    pub digest: [u8; 32],
    pub permissions: FilePermissions,

    pub parent_folder: PathBuf,
    pub file_name: PathBuf,
}

#[derive(Debug)]
enum StaticFileContext {
    NoFile {
        parent_fd: OwnedFd,
    },
    File {
        parent_fd: OwnedFd,
        target_fd: OwnedFd,
        state: FileState,
    },
}

#[derive(Debug)]
pub enum StaticFilePlan {
    Create { parent_fd: OwnedFd },
    Replace { parent_fd: OwnedFd },
    Delete { parent_fd: OwnedFd },
    Noop,
}

fn statxts_to_syst(ts: StatxTimestamp) -> SystemTime {
    let time = Duration::new(ts.tv_sec.abs_diff(0), ts.tv_nsec);

    if ts.tv_sec > 0 {
        UNIX_EPOCH.saturating_add(time)
    } else {
        UNIX_EPOCH.saturating_sub(time)
    }
}

impl StaticFileReconciler {
    #[must_use]
    pub const fn new_in(root: PathBuf) -> Self {
        Self { root }
    }
}

impl StaticFileReconciler {
    pub async fn validate(
        &self,
        spec: StaticFileSpec,
        resource: Option<StaticFileResource>,
    ) -> Result<ValidateResponse<StaticFileDerivedSpec>> {
        if let Some(resource) = resource {
            self.validate_spec_change(&resource, &spec).await?;
        } else {
            self.validate_new_spec(&spec).await?;
        }

        Ok(ValidateResponse {
            derived_spec: self.derive(&spec).await?,
            children: vec![],
            dependencies: vec![],
        })
    }

    pub async fn reconcile(
        &self,
        resource: StaticFileResource,
    ) -> Result<ResourceResponse<FileState>> {
        if let Err(err) = self.validate_new_spec(&resource.spec).await {
            return Ok(ResourceResponse {
                status: Status::Error(format!("{err:#}")),
                state: resource.state,
                children: vec![],
                dependencies: vec![],
            });
        }

        let cx = match self.refresh(&resource).await {
            Ok(v) => v,
            Err(err) => {
                return Ok(ResourceResponse {
                    status: Status::Error(format!("{err:#}")),
                    state: resource.state,
                    children: vec![],
                    dependencies: vec![],
                });
            }
        };

        let state = match &cx {
            StaticFileContext::NoFile { parent_fd: _ } => None,
            StaticFileContext::File {
                parent_fd: _,
                target_fd: _,
                state,
            } => Some(state.clone()),
        };

        let plan = match self.plan(&resource, cx).await {
            Ok(v) => v,
            Err(err) => {
                return Ok(ResourceResponse {
                    status: Status::Error(format!("{err:#}")),
                    state,
                    children: vec![],
                    dependencies: vec![],
                });
            }
        };

        let () = match self.apply(&resource, &plan).await {
            Ok(v) => v,
            Err(err) => {
                return Ok(ResourceResponse {
                    status: Status::Error(format!("{err:#}")),
                    state,
                    children: vec![],
                    dependencies: vec![],
                });
            }
        };

        let new_cx = match self.refresh(&resource).await {
            Ok(v) => v,
            Err(err) => {
                return Ok(ResourceResponse {
                    status: Status::Error(format!("{err:#}")),
                    state,
                    children: vec![],
                    dependencies: vec![],
                });
            }
        };

        let state = match &new_cx {
            StaticFileContext::NoFile { parent_fd: _ } => None,
            StaticFileContext::File {
                parent_fd: _,
                target_fd: _,
                state,
            } => Some(state.clone()),
        };

        let new_plan = match self.plan(&resource, new_cx).await {
            Ok(v) => v,
            Err(err) => {
                return Ok(ResourceResponse {
                    status: Status::Error(format!("{err:#}")),
                    state,
                    children: vec![],
                    dependencies: vec![],
                });
            }
        };

        let status = match new_plan {
            StaticFilePlan::Noop
                if matches!(resource.phase, Phase::Teardown) =>
            {
                Status::Deleted
            }
            StaticFilePlan::Noop => Status::Done,
            StaticFilePlan::Create { parent_fd: _ }
            | StaticFilePlan::Replace { parent_fd: _ }
            | StaticFilePlan::Delete { parent_fd: _ } => Status::NotReady,
        };

        Ok(ResourceResponse {
            status,
            state,
            children: vec![],
            dependencies: vec![],
        })
    }

    async fn validate_new_spec(&self, spec: &StaticFileSpec) -> Result<()> {
        let mut components = spec.path.components();
        let Some(std::path::Component::RootDir) = components.next() else {
            bail!("path is not absolute");
        };

        if components.any(|c| !matches!(c, std::path::Component::Normal(_))) {
            bail!("path should not contain \".\" or \"..\"");
        }

        let normalized = spec
            .path
            .normalize_lexically()
            .context("unable to normalize path")?;

        if normalized.as_os_str() != spec.path.as_os_str() {
            bail!("path is not normalized");
        }

        spec.path
            .strip_prefix(&self.root)
            .context("path is not within permitted root")?;

        Ok(())
    }

    async fn validate_spec_change(
        &self,
        _resource: &StaticFileResource,
        spec: &StaticFileSpec,
    ) -> Result<()> {
        self.validate_new_spec(spec).await
    }

    async fn derive(
        &self,
        spec: &StaticFileSpec,
    ) -> Result<StaticFileDerivedSpec> {
        let digest = sha3::Sha3_256::new()
            .chain_update(spec.content.as_bytes())
            .finalize();
        let mut permissions = FilePermissions::RUSR;
        if spec.readable_by_group {
            permissions = permissions.union(FilePermissions::RGRP);
        }

        if spec.readable_by_others {
            permissions = permissions.union(FilePermissions::ROTH);
        }

        let rel_path = spec.path.strip_prefix(&self.root).context(
            "unexpected error while attempting remove the root prefix",
        )?;

        Ok(StaticFileDerivedSpec {
            digest: digest.into(),
            size: spec
                .content
                .len()
                .try_into()
                .context("unable to compute file size")?,
            permissions,
            file_name: rel_path
                .file_name()
                .map(PathBuf::from)
                .ok_or_else(|| anyhow!("unable to extract file name"))?,
            parent_folder: rel_path
                .parent()
                .map(PathBuf::from)
                .ok_or_else(|| anyhow!("unable to extract parent folder"))?,
        })
    }

    async fn refresh(
        &self,
        resource: &StaticFileResource,
    ) -> Result<StaticFileContext> {
        let root_fd = openat2(
            CWD,
            &self.root,
            OFlags::RDONLY | OFlags::DIRECTORY,
            Mode::empty(),
            ResolveFlags::NO_SYMLINKS,
        )
        .context("unexpected error while opening the root directory")?;

        let parent_fd = if resource.derived_spec.parent_folder.is_empty() {
            root_fd
        } else {
            openat2(
                root_fd,
                &resource.derived_spec.parent_folder,
                OFlags::RDONLY | OFlags::DIRECTORY,
                Mode::empty(),
                ResolveFlags::NO_SYMLINKS | ResolveFlags::BENEATH,
            )
            .context("unexpected error while opening parent directory")?
        };

        let maybe_fd = openat2(
            &parent_fd,
            &resource.derived_spec.file_name,
            OFlags::RDONLY,
            Mode::empty(),
            ResolveFlags::NO_SYMLINKS
                | ResolveFlags::BENEATH
                | ResolveFlags::NO_XDEV,
        )
        .map(Some)
        .or_else(|err| match err {
            Errno::NOENT => Ok(None),
            err => Err(err),
        })
        .context("unexpected error while attempting to open file")?;

        let Some(target_fd) = maybe_fd else {
            return Ok(StaticFileContext::NoFile { parent_fd });
        };

        let stat = statx(
            &target_fd,
            "",
            AtFlags::EMPTY_PATH
                | AtFlags::NO_AUTOMOUNT
                | AtFlags::SYMLINK_NOFOLLOW,
            StatxFlags::empty(),
        )
        .context("unexpected error while attempting to stat the file")?;

        let file_type = match u32::from(stat.stx_mode) & libc::S_IFMT {
            libc::S_IFBLK => FileType::BlockDevice,
            libc::S_IFCHR => FileType::CharacterDevice,
            libc::S_IFDIR => FileType::Directory,
            libc::S_IFIFO => FileType::Fifo,
            libc::S_IFLNK => FileType::Symlink,
            libc::S_IFREG => FileType::Regular,
            libc::S_IFSOCK => FileType::Socket,
            _ => FileType::Unknown,
        };

        let state = FileState {
            atime: statxts_to_syst(stat.stx_atime),
            btime: statxts_to_syst(stat.stx_btime),
            mtime: statxts_to_syst(stat.stx_mtime),
            ctime: statxts_to_syst(stat.stx_ctime),
            permissions: FilePermissions::from_bits_truncate(stat.stx_mode),
            file_type,

            blksize: stat.stx_blksize,
            nlink: stat.stx_nlink,
            uid: stat.stx_uid,
            gid: stat.stx_gid,
            ino: stat.stx_ino,
            size: stat.stx_size,
            blocks: stat.stx_blocks,
            rdev_major: stat.stx_rdev_major,
            rdev_minor: stat.stx_rdev_minor,
            dev_major: stat.stx_dev_major,
            dev_minor: stat.stx_dev_minor,
            mnt_id: stat.stx_mnt_id,
            dio_mem_align: stat.stx_dio_mem_align,
            dio_offset_align: stat.stx_dio_offset_align,
            subvol: stat.stx_subvol,
            atomic_write_unit_min: stat.stx_atomic_write_unit_min,
            atomic_write_unit_max: stat.stx_atomic_write_unit_max,
            atomic_write_segments_max: stat.stx_atomic_write_segments_max,
            dio_read_offset_align: stat.stx_dio_read_offset_align,
            atomic_write_unit_max_opt: stat.stx_atomic_write_unit_max_opt,
        };

        Ok(StaticFileContext::File {
            parent_fd,
            target_fd,
            state,
        })
    }

    async fn plan(
        &self,
        resource: &StaticFileResource,
        cx: StaticFileContext,
    ) -> Result<StaticFilePlan> {
        match (&resource.phase, cx) {
            (
                Phase::Teardown,
                StaticFileContext::File {
                    parent_fd,
                    target_fd: _,
                    state: _,
                },
            ) => Ok(StaticFilePlan::Delete { parent_fd }),

            (Phase::Running, StaticFileContext::NoFile { parent_fd }) => {
                Ok(StaticFilePlan::Create { parent_fd })
            }

            (
                Phase::Running,
                StaticFileContext::File {
                    state,
                    target_fd,
                    parent_fd,
                },
            ) => {
                if !matches!(state.file_type, FileType::Regular) {
                    bail!("target path is occupied by a non-regular file")
                }

                let spec = &resource.spec;
                let derived_spec = &resource.derived_spec;

                if state.uid != getuid().as_raw() {
                    return Ok(StaticFilePlan::Replace { parent_fd });
                }

                match spec.owner_gid {
                    Some(v) => {
                        if state.gid != v {
                            return Ok(StaticFilePlan::Replace { parent_fd });
                        }
                    }
                    None => {
                        if state.gid != getgid().as_raw() {
                            return Ok(StaticFilePlan::Replace { parent_fd });
                        }
                    }
                }

                if state.permissions != derived_spec.permissions {
                    return Ok(StaticFilePlan::Replace { parent_fd });
                }

                if state.size != derived_spec.size {
                    return Ok(StaticFilePlan::Replace { parent_fd });
                }

                let file = std::fs::File::from(target_fd);
                let reader = BufReader::new(file);

                let is_content_equal = reader
                    .bytes()
                    .zip(spec.content.as_bytes())
                    .try_fold(true, |_, (a, b)| {
                        let a = a?;
                        Ok::<_, anyhow::Error>(a == *b)
                    })
                    .context("unable to read existing file")?;

                if is_content_equal {
                    return Ok(StaticFilePlan::Noop);
                }

                Ok(StaticFilePlan::Replace { parent_fd })
            }
            (
                Phase::Shutdown | Phase::Teardown,
                StaticFileContext::NoFile { parent_fd: _ }
                | StaticFileContext::File {
                    parent_fd: _,
                    target_fd: _,
                    state: _,
                },
            ) => Ok(StaticFilePlan::Noop),
        }
    }

    async fn apply(
        &self,
        resource: &StaticFileResource,
        plan: &StaticFilePlan,
    ) -> Result<()> {
        match plan {
            StaticFilePlan::Delete { parent_fd } => {
                self.delete(resource, parent_fd).await
            }
            StaticFilePlan::Create { parent_fd }
            | StaticFilePlan::Replace { parent_fd } => {
                self.create_or_replace(resource, parent_fd).await
            }
            StaticFilePlan::Noop => Ok(()),
        }
    }

    async fn delete(
        &self,
        resource: &StaticFileResource,
        parent_fd: &OwnedFd,
    ) -> Result<()> {
        unlinkat(
            parent_fd,
            &resource.derived_spec.file_name,
            AtFlags::empty(),
        )
        .context("unable to delete file")
    }

    async fn create_or_replace(
        &self,
        resource: &StaticFileResource,
        parent_fd: &OwnedFd,
    ) -> Result<()> {
        let mut tmp_file = openat2(
            parent_fd,
            ".",
            OFlags::WRONLY | OFlags::TMPFILE,
            Mode::WUSR,
            ResolveFlags::NO_SYMLINKS | ResolveFlags::BENEATH,
        )
        .map(std::fs::File::from)
        .context("unable to create temporary file for atomic update")?;

        tmp_file
            .write_all(resource.spec.content.as_bytes())
            .context("unable to write content to working file")?;

        let permissions = Permissions::from_mode(
            resource.derived_spec.permissions.bits().into(),
        );

        tmp_file
            .set_permissions(permissions)
            .context("unable to set permissions on tempfile")?;

        let current_gid = getgid();
        let target_gid =
            resource.spec.owner_gid.map_or_else(getgid, Gid::from_raw);

        let do_change_gid = current_gid != target_gid;

        let needs_chown = do_change_gid;
        if needs_chown {
            std::os::unix::fs::fchown(
                &tmp_file,
                None,
                Some(target_gid.as_raw()),
            )
            .context("unable to chown the tempfile")?;
        }

        tmp_file.sync_all().context("unable to sync working file")?;

        linkat(
            tmp_file.as_fd(),
            "",
            parent_fd,
            &resource.derived_spec.file_name,
            AtFlags::EMPTY_PATH,
        )
        .context("unable to link working file to final destination")?;

        fsync(parent_fd).context("unable to sync root directory")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;
    use std::path::PathBuf;

    use cos_proto_reconciler::{Identity, Key, assert_reconciliation_error};
    use tempfile::{TempDir, tempdir};

    use super::*;

    /*
        Validation (stage 1):
            - Non absolute path (doesn't start with `/`)
            - Absolute path with relative directories (`/something/../whatever`)
            - Absolute path with relative directories escaping (`/something/../../../whatever`)
            - Absolute path with non-normalized directories (`/something/./././whatever`)
            - Absolute path not within the root
        Validation (stage 2):
            - Path only points to an existing **regular** file or points to nothing
            - Filesystem is read-only
        Reconciliation:
            - File exist:
                - With same content
                - With different content
                - With different permissions
            - File does not exist
    */

    mod validation {
        use super::*;

        #[test]
        fn basic_should_succeed() {
            let root = PathBuf::from("/tmp/test");
            let spec = StaticFileSpec {
                path: root.join("test.txt"),
                content: "my-content".to_owned(),
                owner_gid: None,
                readable_by_group: false,
                readable_by_others: false,
            };
            let reconciler = StaticFileReconciler::new_in(root);
            smol::block_on(reconciler.validate_new_spec(&spec)).unwrap();
        }

        #[test]
        fn non_absolute_path_should_fail() {
            let root = PathBuf::from("/");
            let spec = StaticFileSpec {
                path: PathBuf::from("foo/bar/../baz.txt"),
                content: "my-content".to_owned(),
                owner_gid: None,
                readable_by_group: false,
                readable_by_others: false,
            };
            let reconciler = StaticFileReconciler::new_in(root);
            smol::block_on(reconciler.validate_new_spec(&spec)).unwrap_err();
        }

        #[test]
        fn directory_traversal_should_fail() {
            let root = PathBuf::from("/");
            let spec = StaticFileSpec {
                path: PathBuf::from("/foo/bar/../baz.txt"),
                content: "my-content".to_owned(),
                owner_gid: None,
                readable_by_group: false,
                readable_by_others: false,
            };
            let reconciler = StaticFileReconciler::new_in(root);
            smol::block_on(reconciler.validate_new_spec(&spec)).unwrap_err();
        }

        #[test]
        fn escaping_directory_traversal_should_fail() {
            let root = PathBuf::from("/");
            let spec = StaticFileSpec {
                path: PathBuf::from("/foo/bar/../../../../baz.txt"),
                content: "my-content".to_owned(),
                owner_gid: None,
                readable_by_group: false,
                readable_by_others: false,
            };
            let reconciler = StaticFileReconciler::new_in(root);
            smol::block_on(reconciler.validate_new_spec(&spec)).unwrap_err();
        }

        #[test]
        fn non_normalized_path_should_fail() {
            let root = PathBuf::from("/");
            let spec = StaticFileSpec {
                path: PathBuf::from("/foo/./bar/./baz.txt"),
                content: "my-content".to_owned(),
                owner_gid: None,
                readable_by_group: false,
                readable_by_others: false,
            };
            let reconciler = StaticFileReconciler::new_in(root);
            smol::block_on(reconciler.validate_new_spec(&spec)).unwrap_err();
        }

        #[test]
        fn path_not_in_root_should_fail() {
            let root = PathBuf::from("/tmp/test");
            let spec = StaticFileSpec {
                path: PathBuf::from("/foo/bar/baz.txt"),
                content: "my-content".to_owned(),
                owner_gid: None,
                readable_by_group: false,
                readable_by_others: false,
            };
            let reconciler = StaticFileReconciler::new_in(root);
            smol::block_on(reconciler.validate_new_spec(&spec)).unwrap_err();
        }
    }

    mod derivation {
        use super::*;

        #[test]
        fn basic_should_succeed() {
            let root = tempdir().unwrap();
            let reconciler =
                StaticFileReconciler::new_in(root.path().to_path_buf());

            let spec = StaticFileSpec {
                path: root.path().join("test.txt"),
                content: "my-content".to_owned(),
                owner_gid: None,
                readable_by_group: false,
                readable_by_others: false,
            };

            let derived_spec =
                smol::block_on(reconciler.derive(&spec)).unwrap();
            assert_eq!(derived_spec.permissions, FilePermissions::RUSR);
        }

        #[test]
        fn readable_by_group_should_succeed() {
            let root = tempdir().unwrap();
            let reconciler =
                StaticFileReconciler::new_in(root.path().to_path_buf());

            let spec = StaticFileSpec {
                path: root.path().join("test.txt"),
                content: "my-content".to_owned(),
                owner_gid: None,
                readable_by_group: true,
                readable_by_others: false,
            };

            let derived_spec =
                smol::block_on(reconciler.derive(&spec)).unwrap();
            assert_eq!(
                derived_spec.permissions,
                FilePermissions::RUSR | FilePermissions::RGRP
            );
        }

        #[test]
        fn readable_by_other_should_succeed() {
            let root = tempdir().unwrap();
            let reconciler =
                StaticFileReconciler::new_in(root.path().to_path_buf());

            let spec = StaticFileSpec {
                path: root.path().join("test.txt"),
                content: "my-content".to_owned(),
                owner_gid: None,
                readable_by_group: false,
                readable_by_others: true,
            };

            let derived_spec =
                smol::block_on(reconciler.derive(&spec)).unwrap();
            assert_eq!(
                derived_spec.permissions,
                FilePermissions::RUSR | FilePermissions::ROTH
            );
        }

        #[test]
        fn readable_by_everyone_should_succeed() {
            let root = tempdir().unwrap();
            let reconciler =
                StaticFileReconciler::new_in(root.path().to_path_buf());

            let spec = StaticFileSpec {
                path: root.path().join("test.txt"),
                content: "my-content".to_owned(),
                owner_gid: None,
                readable_by_group: true,
                readable_by_others: true,
            };

            let derived_spec =
                smol::block_on(reconciler.derive(&spec)).unwrap();
            assert_eq!(
                derived_spec.permissions,
                FilePermissions::RUSR
                    | FilePermissions::RGRP
                    | FilePermissions::ROTH
            );
        }
    }

    mod refresh {

        use super::*;

        #[test]
        fn basic_should_succeed() {
            let root = tempdir().unwrap();
            let reconciler =
                StaticFileReconciler::new_in(root.path().to_path_buf());

            let spec = StaticFileSpec {
                path: root.path().join("test.txt"),
                content: "my-content".to_owned(),
                owner_gid: None,
                readable_by_group: false,
                readable_by_others: false,
            };
            let derived_spec =
                smol::block_on(reconciler.derive(&spec)).unwrap();

            let file = StaticFileResource {
                id: Identity::Static(Key {
                    schema: String::new(),
                    name: None,
                }),
                phase: Phase::Running,
                status: Status::Unknown,
                spec,
                derived_spec,
                state: None,
                children: vec![],
                dependencies: vec![],
                dependents: vec![],
            };
            let refreshed = smol::block_on(reconciler.refresh(&file)).unwrap();
            assert_matches!(
                refreshed,
                StaticFileContext::NoFile { parent_fd: _ }
            );
        }

        #[test]
        fn non_regular_file_should_succeed() {
            let reconciler =
                StaticFileReconciler::new_in(PathBuf::from("/"));

            let spec = StaticFileSpec {
                path: PathBuf::from("/dev/null"),
                content: "my-content".to_owned(),
                owner_gid: None,
                readable_by_group: false,
                readable_by_others: false,
            };
            let derived_spec =
                smol::block_on(reconciler.derive(&spec)).unwrap();

            let file = StaticFileResource {
                id: Identity::Static(Key {
                    schema: String::new(),
                    name: None,
                }),
                phase: Phase::Running,
                status: Status::Unknown,
                spec,
                derived_spec,
                state: None,
                children: vec![],
                dependencies: vec![],
                dependents: vec![],
            };

            let result = smol::block_on(reconciler.refresh(&file)).unwrap();
            let StaticFileContext::File {
                state,
                parent_fd: _,
                target_fd: _,
            } = result
            else {
                panic!("/dev/null should exist");
            };
            assert_matches!(state.file_type, FileType::CharacterDevice);
        }
    }

    mod reconciliation {
        use super::*;

        fn create_ok_resource()
        -> (TempDir, StaticFileReconciler, StaticFileResource) {
            let root = tempdir().unwrap();
            let reconciler =
                StaticFileReconciler::new_in(root.path().to_path_buf());

            let spec = StaticFileSpec {
                path: root.path().join("test.txt"),
                content: "my-content".to_owned(),
                owner_gid: None,
                readable_by_group: false,
                readable_by_others: false,
            };
            let derived_spec =
                smol::block_on(reconciler.derive(&spec)).unwrap();

            let file = StaticFileResource {
                id: Identity::Static(Key {
                    schema: String::new(),
                    name: None,
                }),
                phase: Phase::Running,
                status: Status::Unknown,
                spec,
                derived_spec,
                state: None,
                children: vec![],
                dependencies: vec![],
                dependents: vec![],
            };

            (root, reconciler, file)
        }

        #[test]
        fn basic_should_succeed() {
            let (_root, reconciler, file) = create_ok_resource();

            let result =
                smol::block_on(reconciler.reconcile(file.clone())).unwrap();
            assert_matches!(result.status, Status::Done);

            let state = result.state.unwrap();
            assert_eq!(state.file_type, FileType::Regular);

            let content = std::fs::read_to_string(&file.spec.path).unwrap();
            assert_eq!(&content, &file.spec.content);
        }

        #[test]
        fn existing_should_succeed() {
            let (_root, reconciler, mut file) = create_ok_resource();
            let result =
                smol::block_on(reconciler.reconcile(file.clone())).unwrap();
            assert_matches!(result.status, Status::Done);
            assert!(std::fs::exists(&file.spec.path).unwrap());

            file.status = Status::Unknown;
            let result =
                smol::block_on(reconciler.reconcile(file.clone())).unwrap();
            assert_matches!(result.status, Status::Done);

            let content = std::fs::read_to_string(&file.spec.path).unwrap();
            assert_eq!(&content, &file.spec.content);
        }

        #[test]
        fn delete_should_succeed() {
            let (mut root, reconciler, mut file) = create_ok_resource();
            root.disable_cleanup(true);

            let result =
                smol::block_on(reconciler.reconcile(file.clone())).unwrap();
            assert_matches!(result.status, Status::Done);
            assert!(std::fs::exists(&file.spec.path).unwrap());

            file.phase = Phase::Teardown;
            let result =
                smol::block_on(reconciler.reconcile(file.clone())).unwrap();
            assert_matches!(result.status, Status::Deleted);

            assert!(!std::fs::exists(&file.spec.path).unwrap());
        }

        #[test]
        fn non_regular_file_should_fail() {
            let reconciler =
                StaticFileReconciler::new_in(PathBuf::from("/"));

            let spec = StaticFileSpec {
                path: PathBuf::from("/dev/null"),
                content: "my-content".to_owned(),
                owner_gid: None,
                readable_by_group: false,
                readable_by_others: false,
            };
            let derived_spec =
                smol::block_on(reconciler.derive(&spec)).unwrap();

            let file = StaticFileResource {
                id: Identity::Static(Key {
                    schema: String::new(),
                    name: None,
                }),
                phase: Phase::Running,
                status: Status::Unknown,
                spec,
                derived_spec,
                state: None,
                children: vec![],
                dependencies: vec![],
                dependents: vec![],
            };
            let result =
                smol::block_on(reconciler.reconcile(file)).unwrap();
            assert_reconciliation_error!(
                result.status,
                "target path is occupied by a non-regular file"
            );
        }
    }
}
