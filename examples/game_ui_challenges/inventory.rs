use bevy::prelude::*;
use rxy_bevy::navigation::RxyKeyboardNavigationPlugin;
use rxy_ui::prelude::*;
use std::borrow::Cow;

use async_channel::Receiver;
use bevy::asset::AssetLoader;
use bevy::utils::OnDrop;
use std::fmt::Debug;
use std::ops::Deref;

mod components;

use components::*;
use hooked_collection::{HookVec, HookedVec, VecOperation};
use rxy_bevy::vec_data_source::use_hooked_vec_resource_source;
use rxy_core::utils::SyncCell;
use rxy_core::NodeTree;

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins,
        RxyPlugin::default(),
        RxyStyleSheetPlugin::default(),
        RxyKeyboardNavigationPlugin::default(),
    ))
    .init_resource::<DraggingInventoryItem>()
    .init_resource::<InventoryIsDragging>()
    .add_systems(Startup, setup);

    app.run();
}

const INVENTORY_WIDTH: u16 = 10;
const INVENTORY_HEIGHT: u16 = 5;

#[derive(Resource)]
pub struct SampleItems(Vec<Item>);

fn setup(mut world: &mut World) {
    world.spawn(Camera2dBundle::default());

    let (sender, receiver) = async_channel::unbounded();
    let mut items =
        vec![InventoryItemContainer::default(); (INVENTORY_WIDTH * INVENTORY_HEIGHT).into()];

    {
        let asset_server = world.resource_mut::<AssetServer>();

        let sample_items = vec![
            Item {
                name: "Item 1".to_string(),
                icon: asset_server.load::<Image>("items/0.png"),
            },
            Item {
                name: "Item 2".to_string(),
                icon: asset_server.load::<Image>("items/1.png"),
            },
            Item {
                name: "Item 3".to_string(),
                icon: asset_server.load::<Image>("items/2.png"),
            },
            Item {
                name: "Item 4".to_string(),
                icon: asset_server.load::<Image>("items/3.png"),
            },
            Item {
                name: "Item 5".to_string(),
                icon: asset_server.load::<Image>("items/4.png"),
            },
        ];
        items[0] = InventoryItemContainer::new(sample_items[0].clone(), 2);
        items[4] = InventoryItemContainer::new(sample_items[4].clone(), 1);
        items[10] = InventoryItemContainer::new(sample_items[0].clone(), 1);
        items[20] = InventoryItemContainer::new(sample_items[1].clone(), 10);
        items[33] = InventoryItemContainer::new(sample_items[1].clone(), 2);
        items[35] = InventoryItemContainer::new(sample_items[3].clone(), 4);
        items[8] = InventoryItemContainer::new(sample_items[2].clone(), 6);
        world.insert_resource(SampleItems(sample_items));
    }
    world.insert_resource(InventoryItems(HookedVec::from_vec(items, sender)));
    world.insert_resource(InventoryItemsOpReceiver(receiver));

    world.spawn_view_on_root(game_ui());
}

#[derive(TypedStyle)]
struct FocusStyle;

#[derive(Clone, Debug)]
pub struct Item {
    pub name: String,
    pub icon: Handle<Image>,
}

#[derive(Clone, Debug)]
pub struct InventoryItem {
    pub item: Item,
    pub count: u32,
}

#[derive(Clone, Debug, PropValueWrapper, Default)]
pub struct InventoryItemContainer(Option<InventoryItem>);

impl InventoryItemContainer {
    pub fn new(item: Item, count: u32) -> Self {
        Self(Some(InventoryItem { item, count }))
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct InventoryItems(
    HookedVec<InventoryItemContainer, Sender<VecOperation<InventoryItemContainer>>>,
);

#[derive(Resource)]
pub struct InventoryItemsOpReceiver(Receiver<VecOperation<InventoryItemContainer>>);

fn game_ui() -> impl IntoView<BevyRenderer> {
    div()
        .p(20)
        .size_screen()
        .flex()
        .flex_col()
        .center()
        .children((
            ("New:",),
            view_builder(|ctx: ViewCtx<BevyRenderer>, _| {
                let receiver = ctx
                    .world
                    .remove_resource::<InventoryItemsOpReceiver>()
                    .unwrap();
                let source = use_hooked_vec_resource_source::<InventoryItems>(receiver.0);
                div()
                    .bg_color(Color::GRAY)
                    .grid()
                    .gap(10)
                    .padding(10)
                    .grid_template_columns(vec![RepeatedGridTrack::auto(INVENTORY_WIDTH)])
                    .grid_template_rows(vec![RepeatedGridTrack::auto(INVENTORY_HEIGHT)])
                    .children(x_iter_source(
                        source,
                        |item: Cow<InventoryItemContainer>, index: usize| {
                            inventory_item_view(item.into_owned(), index)
                        },
                    ))
            }),
        ))
}

#[derive(Resource, Default)]
pub struct DraggingInventoryItem {
    // item: InventoryItemContainer,
    // is_drag: RwSignal<bool>,
    delta: Vec2,
    index: usize,
    view_key: Option<SyncCell<OnDrop<Box<dyn FnOnce() + Send>>>>,
}

impl DraggingInventoryItem {
    pub fn reset(&mut self) {
        self.delta = Default::default();
        self.view_key = None;
    }
}

#[derive(Resource, Default, Deref)]
pub struct InventoryIsDragging(bool);

#[derive(ElementSchema)]
pub struct InventoryItemView {
    item: Required<ReadSignal<InventoryItemContainer>>,
    index: Required<Static<usize>>,
}

impl SchemaElementView<BevyRenderer> for InventoryItemView {
    fn view(self) -> impl IntoElementView<BevyRenderer> {
        let InventoryItemView {
            item: Required(item),
            index: Required(Static(index)),
        } = self;

        let root = div()
            .size(50)
            .style((
                x().relative()
                    .bg_color(Color::WHITE)
                    .border(1)
                    .border_color(Color::BLACK),
                x_hover().bg_color(Color::GRAY),
            ))
            // .style(x_res(|is_dragging: &InventoryIsDragging| {
            //     is_dragging.0.then_some(x_hover().border_color(Color::BLUE))
            // }))
            ;
        root.children(rx(move || {
            if let Some(item) = item.get().0 {
                fn item_view(InventoryItem { item, count }: InventoryItem) -> impl ElementView<BevyRenderer> {
                    div()
                        .size_full()
                        .absolute()
                        .children((
                            img().m(8).src(item.icon),
                            span(count.to_string())
                                .text_color(Color::BLUE)
                                .font_size(18)
                                .absolute()
                                .top(1)
                                .right(1),
                        ))
                }
                let events = ()
                    .on_pointer_drag(
                        move |e: Res<ListenerInputPointerDrag>,
                              mut dargging: ResMut<DraggingInventoryItem>| {
                            dargging.delta += e.delta;
                        },
                    )
                    .on_pointer_drag_end(move |mut dragging: ResMut<DraggingInventoryItem>, mut is_dragging: ResMut<InventoryIsDragging>| {
                        dragging.reset();
                        *is_dragging = InventoryIsDragging(false);
                    })
                    .on_pointer_drop(
                        move |e: Res<ListenerInputPointerDrop>,
                              mut dragging: ResMut<DraggingInventoryItem>,mut is_dragging: ResMut<InventoryIsDragging>,
                              mut inventory_items: ResMut<InventoryItems>| {
                            println!("swap {:?} {:?}",index,dragging.index);
                            inventory_items.swap(index, dragging.index);
                            dragging.reset();
                            *is_dragging = InventoryIsDragging(false);
                        },
                    )
                    .on_pointer_drag_start({
                        let item = item.clone();
                        // let item = InventoryItemContainer::new(item.clone(), count);
                        // move |mut draggging: ResMut<DraggingInventoryItem>,
                        //       mut is_dragging: ResMut<InventoryIsDragging>| {
                        move |world: &mut World| {
                            *world.resource_mut::<InventoryIsDragging>() = InventoryIsDragging(true);
                            let e =world.resource::<ListenerInputPointerDragStart>();
                            let parent = world.get_parent(&e.listener()).unwrap();
                            let view_key =world.spawn_view(
                                into_view(item_view(item.clone())
                                    .z(1)
                                    .member(
                                        x_res(move |dragging: &DraggingInventoryItem| {
                                            ().left(dragging.delta.x).top(dragging.delta.y)
                                        })
                                    )),
                                 move |_| parent
                            );
                            // world.spawn_rxy_ui()
                            // is_drag.set(true);
                            let cmd_sender = world.resource::<CmdSender>().clone();
                            *world.resource_mut::<DraggingInventoryItem>() = DraggingInventoryItem {
                                delta: Vec2::default(),
                                index,
                                view_key: Some(SyncCell::new(OnDrop::new(Box::new(move || {
                                    cmd_sender.add(|world:&mut World| {
                                        view_key.remove(world)
                                    });
                                }))))
                            };
                        }
                    })
                    ;
                into_view(item_view(item)
                    .member(events))
                    .either_left()
            } else {
                ().either_right()
            }
        }))
    }
}
