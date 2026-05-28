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

// pub fn spec_inner(&self) -> &T {
// self.spec.inner()
// }
//
// pub fn state_inner(&self) -> Option<&T::State> {
// self.state.inner()
// }
// pub fn spec_inner_mut(&mut self) -> &mut T {
// self.spec.inner_mut()
// }
//
// pub fn state_inner_mut(&mut self) -> Option<&mut T::State> {
// self.state.inner_mut()
// }

#[macro_export]
macro_rules! delegate_to_meta {
    (@ro-, $t:path) => {
        pub const fn id(&self) -> &Identity {
            self.meta().id()
        }

        pub const fn children(&self) -> &HashSet<Identity> {
            self.meta().children()
        }

        pub const fn spec(&self) -> &ResourceSpec<$t> {
            self.meta().spec()
        }

        pub const fn state(&self) -> &ResourceState<$t> {
            self.meta().state()
        }

        pub fn spec_inner(&self) -> &T {
            self.meta().spec_inner()
        }

        pub fn state_inner(&self) -> Option<&T::State> {
            self.meta().state_inner()
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

        pub const fn spec_mut(&mut self) -> &mut ResourceSpec<$t> {
            self.meta_mut().spec_mut()
        }

        pub const fn state_mut(&mut self) -> &mut ResourceState<$t> {
            self.meta_mut().state_mut()
        }

        pub fn spec_inner_mut(&mut self) -> &mut T {
            self.meta_mut().spec_inner_mut()
        }

        pub fn state_inner_mut(&mut self) -> Option<&mut T::State> {
            self.meta_mut().state_inner_mut()
        }
    };

    (@rw, $t:path) => {
        delegate_to_meta!(@ro, $t);

        pub const fn meta_mut(&mut self) -> &mut ResourceMeta<$t> {
            &mut self.meta
        }
    }
}
