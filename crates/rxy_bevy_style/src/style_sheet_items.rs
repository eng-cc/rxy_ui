use std::iter::once;
use bevy_ecs::all_tuples;
use rxy_bevy::{BevyRenderer, ViewAttr};
use rxy_bevy_element::{ElementUnitAttr, smallbox};
use rxy_core::Renderer;
use rxy_style::StyleSheetCtx;
use crate::plugin::StyleItemValue;

pub trait StyleSheetItems<R>: Send + 'static
where
    R: Renderer,
{
    fn iter(
        self,
        ctx: StyleSheetCtx<R>,
    ) -> impl Iterator<Item = StyleItemValue> + 'static;
}

impl<EA> StyleSheetItems<BevyRenderer> for ViewAttr<EA>
where
    EA: ElementUnitAttr,
{
    #[inline(always)]
    fn iter(
        self,
        _ctx: StyleSheetCtx<BevyRenderer>,
    ) -> impl Iterator<Item = StyleItemValue> + 'static {
        once(StyleItemValue{
            attr_id: EA::INDEX, 
            value: smallbox!(self.0)
        })
    }
}

macro_rules! impl_style_sheet_items_for_tuple {
    ($($t:ident),*) => {
        #[allow(non_snake_case)]
        impl<R, $($t),*> StyleSheetItems<R> for ($($t,)*)
        where
            R: Renderer,
            $($t: StyleSheetItems<R>),*
        {
            #[inline(always)]
            fn iter(
                self,
                _ctx: StyleSheetCtx<R>,
            ) -> impl Iterator<Item = StyleItemValue> + 'static {
                let ($($t,)*) = self;
                core::iter::empty()
                $(
                    .chain($t.iter(StyleSheetCtx{
                        inline_style_sheet_index: _ctx.inline_style_sheet_index,
                        shared_style_sheet_index: _ctx.shared_style_sheet_index,
                        world: unsafe {&mut *(_ctx.world as *mut _)},
                        node_id: _ctx.node_id.clone(),
                    }))
                )*
            }
        }
    };
}
all_tuples!(impl_style_sheet_items_for_tuple, 0, 12, T);
