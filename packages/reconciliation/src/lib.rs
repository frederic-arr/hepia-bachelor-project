// struct InterfaceResource {
//     spec: InterfaceSpec,
//     // status: InterfaceStatus,
// }

// struct InterfaceSpec {
//     name: String,
//     kind: String,
//     state: InterfaceState,
// }

// // struct InterfaceStatus {

// // }

// enum InterfaceState {
//     Up,
//     Down,
// }

// struct InterfaceReconciler;

// impl InterfaceReconciler {
//     pub fn reconcile(&mut self) {
//         // 1. Get desired state
//         // 2. Get current state
//         // 3. Compare and act
//     }
// }

// // struct Reconciler {
// //     resources: Vec<InterfaceSpec>,
// // }

// // impl Reconciler {
// //     fn reconcile(&mut self) {
// //         for resource in &mut self.resources {

// //         }
// //     }
// // }

// // use std::io::{ErrorKind, Read, Write};
// // use std::path::PathBuf;

// // #[derive(Debug, Clone)]
// // struct FileSpec {
// //     path: PathBuf,
// //     content: String,
// // }

// // #[derive(Debug, Clone)]
// // enum FileStatus {
// //     Error,
// //     DoesNotExist,
// //     ContentMismatch { actual_content: String },
// //     Ready,
// // }

// // impl FileSpec {
// //     pub fn new(path: PathBuf, content: String) -> Self {
// //         Self { path, content }
// //     }

// //     pub fn check(&self) -> FileStatus {
// //         match std::fs::read_to_string(&self.path) {
// //             Ok(actual_content) if actual_content == self.content => {
// //                 FileStatus::Ready
// //             }
// //             Ok(actual_content) => {
// //                 FileStatus::ContentMismatch { actual_content }
// //             }
// //             Err(err) if err.kind() == ErrorKind::NotFound => {
// //                 FileStatus::DoesNotExist
// //             }
// //             _ => FileStatus::Error,
// //         }
// //     }

// //     pub fn sync(&self) {
// //         match self.check() {
// //             FileStatus::DoesNotExist => {
// //                 std::fs::write(&self.path, &self.content).unwrap();
// //             }
// //             FileStatus::ContentMismatch { .. } => {
// //                 std::fs::write(&self.path, &self.content).unwrap();
// //             }
// //             FileStatus::Error => {}
// //             FileStatus::Ready => {}
// //         }
// //     }
// // }

// // pub fn add(left: u64, right: u64) -> u64 {
// //     left + right
// // }

// // #[cfg(test)]
// // mod tests {
// //     use std::env::temp_dir;

// //     use super::*;

// //     #[test]
// //     fn create_missing_files() {
// //         let root_dir = temp_dir();
// //         dbg!(&root_dir);
// //         let base_names = vec!["a", "b", "c"];
// //         let test_files = base_names
// //             .iter()
// //             .cloned()
// //             .map(|c| FileSpec::new(root_dir.join(c), c.to_string()))
// //             .collect::<Vec<_>>();

// //         for file in &test_files {
// //             file.sync();
// //         }

// //         for file in base_names {
// //             assert_eq!(
// //                 std::fs::read_to_string(root_dir.join(file)).unwrap(),
// //                 file
// //             );
// //         }

// //         for file in &test_files {
// //             file.sync();
// //         }

// //         for file in base_names {
// //             assert_eq!(
// //                 std::fs::read_to_string(root_dir.join(file)).unwrap(),
// //                 file
// //             );
// //         }
// //     }
// // }
