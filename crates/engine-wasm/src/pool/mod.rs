macro_rules! soa_pool {
    (
        pool $PoolName:ident, spawn $SpawnName:ident {
            $($field:ident : $field_type:ty),* $(,)?
        }
    ) => {
        #[derive(Clone)]
        pub struct $PoolName {
            $(pub $field: Vec<$field_type>,)*
        }

        pub struct $SpawnName {
            $(pub $field: $field_type,)*
        }

        impl $PoolName {
            pub fn new() -> Self {
                Self {
                    $($field: Vec::new(),)*
                }
            }

            pub fn with_capacity(capacity: usize) -> Self {
                Self {
                    $($field: Vec::with_capacity(capacity),)*
                }
            }

            pub fn len(&self) -> usize {
                soa_pool!(@first_len self, $($field),*)
            }

            pub fn clear(&mut self) {
                $(self.$field.clear();)*
            }

            pub fn push(&mut self, item: $SpawnName) {
                $(self.$field.push(item.$field);)*
            }

            pub fn swap_remove(&mut self, index: usize) {
                $(self.$field.swap_remove(index);)*
            }
        }
    };

    (@first_len $self:ident, $first:ident $(, $rest:ident)*) => {
        $self.$first.len()
    };
}

pub mod bullet;
pub mod generator;
pub mod helper;
pub mod object;
