use std::{any::Any, collections::HashMap};

use super::{views::Flex, ViewId};

pub trait WidthProperty: 'static {
    const WIDTH: f32;
}

pub trait HeightProperty: 'static {
    const HEIGHT: f32;
}

pub trait FilledProperty: 'static {
    const FILLED: char;
    const CROSS: char = Self::FILLED;
}

pub trait UnfilledProperty: 'static {
    const UNFILLED: char;
}

#[derive(Default)]
pub struct Properties {
    list: Vec<Box<dyn Any>>,
    local: HashMap<ViewId, Vec<Box<dyn Any>>>,
}

impl Properties {
    pub fn with<P: 'static>(mut self, item: P) -> Self {
        self.insert(item);
        self
    }

    pub fn with_default<P: 'static + Default>(mut self) -> Self {
        self.insert_default::<P>();
        self
    }
}

impl Properties {
    pub fn width<T: WidthProperty>(&mut self) -> f32 {
        struct Width<T: WidthProperty> {
            value: f32,
            _marker: std::marker::PhantomData<T>,
        }
        self.get_or_insert_with::<Width<T>>(|| Width {
            value: T::WIDTH,
            _marker: std::marker::PhantomData,
        })
        .value
    }

    pub fn height<T: HeightProperty>(&mut self) -> f32 {
        struct Height<T: HeightProperty> {
            value: f32,
            _marker: std::marker::PhantomData<T>,
        }
        self.get_or_insert_with::<Height<T>>(|| Height {
            value: T::HEIGHT,
            _marker: std::marker::PhantomData,
        })
        .value
    }

    pub fn filled<T: FilledProperty>(&mut self) -> char {
        struct Filled<T: FilledProperty> {
            value: char,
            _marker: std::marker::PhantomData<T>,
        }
        self.get_or_insert_with::<Filled<T>>(|| Filled {
            value: T::FILLED,
            _marker: std::marker::PhantomData,
        })
        .value
    }

    // TODO this is a bad name
    pub fn filled_cross<T: FilledProperty>(&mut self) -> char {
        struct FilledCross<T: FilledProperty> {
            value: char,
            _marker: std::marker::PhantomData<T>,
        }
        self.get_or_insert_with::<FilledCross<T>>(|| FilledCross {
            value: T::CROSS,
            _marker: std::marker::PhantomData,
        })
        .value
    }

    pub fn unfilled<T: UnfilledProperty>(&mut self) -> char {
        struct Unfilled<T: UnfilledProperty> {
            value: char,
            _marker: std::marker::PhantomData<T>,
        }
        self.get_or_insert_with::<Unfilled<T>>(|| Unfilled {
            value: T::UNFILLED,
            _marker: std::marker::PhantomData,
        })
        .value
    }

    pub fn flex(&mut self, id: ViewId) -> Option<Flex> {
        self.get_for(id).copied()
    }
}

impl Properties {
    pub fn clear_locals(&mut self) {
        self.local.clear();
    }

    pub fn remove_all_for_id(&mut self, id: ViewId) {
        self.local.remove(&id);
    }

    pub fn get_for<P: 'static>(&self, id: ViewId) -> Option<&P> {
        self.local
            .get(&id)?
            .iter()
            .find_map(|c| c.downcast_ref::<P>())
    }

    pub fn get_or_default_for<P: 'static + Default>(&mut self, id: ViewId) -> &P {
        self.get_or_insert_with_for(P::default, id)
    }

    pub fn get_or_insert_for<P: 'static>(&mut self, value: P, id: ViewId) -> &P {
        self.get_or_insert_with_for(|| value, id)
    }

    pub fn get_or_insert_with_for<P: 'static>(
        &mut self,
        value: impl FnOnce() -> P,
        id: ViewId,
    ) -> &P {
        let Some(index) = self.get_index_for::<P>(id) else {
            let item = value();
            self.insert(item);
            return self.local[&id].last().unwrap().downcast_ref::<P>().unwrap();
        };
        self.local[&id][index].downcast_ref::<P>().unwrap()
    }

    pub fn insert_for<P: 'static + std::fmt::Debug>(&mut self, item: P, id: ViewId) {
        match self.get_index_for::<P>(id) {
            Some(index) => self.local.entry(id).or_default()[index] = Box::new(item),
            None => self.local.entry(id).or_default().push(Box::new(item)),
        }
    }

    pub fn insert_default_for<P: 'static + Default + std::fmt::Debug>(&mut self, id: ViewId) {
        self.insert_for(P::default(), id);
    }

    fn get_index_for<P: 'static>(&self, id: ViewId) -> Option<usize> {
        self.local.get(&id)?.iter().position(|item| item.is::<P>())
    }
}

impl Properties {
    pub fn insert<P: 'static>(&mut self, item: P) {
        match self.get_index::<P>() {
            Some(index) => self.list[index] = Box::new(item),
            None => self.list.push(Box::new(item)),
        }
    }

    pub fn insert_default<P: 'static + Default>(&mut self) {
        self.insert(P::default());
    }

    pub fn get<P: 'static>(&self) -> Option<&P> {
        self.list.iter().find_map(|c| c.downcast_ref::<P>())
    }

    pub fn get_or_default<P: 'static + Default>(&mut self) -> &P {
        self.get_or_insert_with(P::default)
    }

    pub fn get_or_insert<P: 'static>(&mut self, value: P) -> &P {
        self.get_or_insert_with(|| value)
    }

    pub fn get_or_insert_with<P: 'static>(&mut self, value: impl FnOnce() -> P) -> &P {
        let Some(index) = self.get_index::<P>() else {
            let item = value();
            self.insert(item);
            return self.list.last().unwrap().downcast_ref::<P>().unwrap();
        };

        self.list[index].downcast_ref::<P>().unwrap()
    }

    pub fn remove<P: 'static>(&mut self) -> bool {
        let len = self.list.len();
        self.list.retain(|c| !c.is::<P>());
        len != self.list.len()
    }

    fn get_index<P: 'static>(&self) -> Option<usize> {
        self.list.iter().position(|item| item.is::<P>())
    }
}
