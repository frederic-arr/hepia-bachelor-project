#[macro_export]
macro_rules! impl_try_from_opt {
    ($src:ty => $dst:ty) => {
        impl TryFrom<Option<$src>> for $dst {
            type Error = String;

            fn try_from(value: Option<$src>) -> Result<Self, Self::Error> {
                value
                    .ok_or_else(|| format!("{} is required", stringify!($src)))?
                    .try_into()
            }
        }
    };
}

#[macro_export]
macro_rules! impl_try_from_opt_bounds {
    ($src:ty => $dst:ident) => {
        impl<T> TryFrom<Option<$src>> for $dst<T>
        where
            T: Specification,
        {
            type Error = String;

            fn try_from(value: Option<$src>) -> Result<Self, Self::Error> {
                value
                    .ok_or_else(|| format!("{} is required", stringify!($src)))?
                    .try_into()
            }
        }
    };
}

#[macro_export]
macro_rules! delegate_to_meta {
    (@ro-, $t:path) => {
        pub const fn id(&self) -> &Identity {
            self.meta().id()
        }

        pub const fn children(&self) -> &HashSet<Identity> {
            self.meta().children()
        }

        pub const fn status(&self) -> &ResourceStatus {
            self.meta().status()
        }

        pub const fn spec(&self) -> &$t {
            self.meta().spec()
        }

        pub const fn state(&self) -> Option<&<$t>::State> {
            self.meta().state()
        }

        pub const fn state_opt(&self) -> &Option<<$t>::State> {
            self.meta().state_opt()
        }
    };

    (@ro, $t:path) => {
        pub const fn meta(&self) -> &ResourceMeta<$t> {
            &self.meta
        }

        delegate_to_meta!(@ro-, $t);
    };

    (@rw-, $t:path) => {
        delegate_to_meta!(@ro-, $t);

        pub const fn children_mut(&mut self) -> &mut HashSet<Identity> {
            self.meta_mut().children_mut()
        }

        pub const fn status_mut(&mut self) -> &mut ResourceStatus {
            self.meta_mut().status_mut()
        }

        pub const fn spec_mut(&mut self) -> &mut $t {
            self.meta_mut().spec_mut()
        }

        pub const fn state_mut(&mut self) -> Option<&mut <$t>::State> {
            self.meta_mut().state_mut()
        }

        pub const fn state_opt_mut(&mut self) -> &mut Option<<$t>::State> {
            self.meta_mut().state_opt_mut()
        }
    };

    (@rw, $t:path) => {
        delegate_to_meta!(@ro, $t);

        pub const fn meta_mut(&mut self) -> &mut ResourceMeta<$t> {
            &mut self.meta
        }
    }
}
