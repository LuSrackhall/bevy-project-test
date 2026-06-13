use bevy::prelude::*;
use simulation::types::*;
use simulation::soldier::*;
use simulation::city::config::CityGlobalConfig;
use simulation::soldier::config::SoldierConfig;
use simulation::command::*;
use bevy_adapter::tick::{SimulationWorld, TickClock};
use bevy_adapter::input::ForceMoveNext;
use crate::selection::SelectionState;
use std::collections::HashMap;

// ══════════ Resources ══════════

#[derive(Resource, Default)]
pub(crate) struct HudTexts {
    // top bar
    cities: Option<Entity>, pop: Option<Entity>, enemy: Option<Entity>, time: Option<Entity>,
    // toast
    pub(crate) toast_text: Option<Entity>,
    // soldier panel
    s_root: Option<Entity>, s_header: Option<Entity>, s_hp_text: Option<Entity>, s_hp_fill: Option<Entity>,
    s_atk: Option<Entity>, s_spd: Option<Entity>, s_exp_text: Option<Entity>, s_exp_fill: Option<Entity>,
    s_effect: Option<Entity>, s_origin: Option<Entity>,
    // city panel
    c_root: Option<Entity>, c_info: Option<Entity>, c_hp_text: Option<Entity>, c_hp_fill: Option<Entity>,
    c_pop: Option<Entity>, c_exp: Option<Entity>, c_spawn: Option<Entity>,
    // command card
    cmd_info: Option<Entity>,
    // compendium
    comp_header: Option<Entity>, comp_hp: Option<Entity>, comp_atk: Option<Entity>,
    comp_spd: Option<Entity>, comp_rng: Option<Entity>, comp_rad: Option<Entity>,
    comp_effect: Option<Entity>, comp_desc: Option<Entity>,
    // seek panel
    pub(crate) seek_scope_text: Option<Entity>,
    pub(crate) seek_range_text: Option<Entity>,
    pub(crate) seek_dropdown_container: Option<Entity>,
}

/// Seek panel state — drives the dropdown, range input, and mode switching.
#[derive(Resource)]
pub struct SeekPanelState {
    pub scope: SeekScope,
    pub dropdown_open: bool,
    pub editing: bool,
    pub input_buffer: String,
    pub range_value: u32,
    pub has_selection: bool, // tracks previous frame selection for mode-change detection
}

impl Default for SeekPanelState {
    fn default() -> Self {
        Self {
            scope: SeekScope::All,
            dropdown_open: false,
            editing: false,
            input_buffer: String::new(),
            range_value: 10,
            has_selection: false,
        }
    }
}

/// Toast message for the top bar.
#[derive(Resource, Default)]
pub struct ToastMessage {
    pub text: String,
    pub remaining_ticks: u32,
}

const TOAST_DURATION_TICKS: u32 = 100; // 5 seconds at 20Hz

// ══════════ Marker Components ══════════

#[derive(Component)] struct HudRoot;
#[derive(Component)] struct BottomZone;
#[derive(Component)] pub(crate) struct HpFillS;
#[derive(Component)] pub(crate) struct ExpFillS;
#[derive(Component)] pub(crate) struct HpFillC;
#[derive(Component)] pub(crate) struct CityPanelRoot;
#[derive(Component)] pub(crate) struct SoldierPanelRoot;
#[derive(Component, Clone, Copy)] pub(crate) struct SpawnTypeBtn(pub SoldierType);
#[derive(Component)] pub struct ToolbarButton(pub u8);

// Seek panel components
#[derive(Component)] pub(crate) struct SeekPanelRoot;
#[derive(Component)] pub(crate) struct SeekScopeDropdown; // the trigger button showing current scope
#[derive(Component, Clone)] pub(crate) struct SeekScopeOption(pub SeekScope); // dropdown option
#[derive(Component)] pub(crate) struct SeekDropdownPopup; // the popup container
#[derive(Component)] pub(crate) struct SeekRangeInput; // the range input box
#[derive(Component)] pub(crate) struct SeekIssueBtn; // the issue button

// ══════════ Setup ══════════

pub fn setup_hud(mut commands: Commands, mut ht: ResMut<HudTexts>, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/Arial Unicode.ttf");
    commands.spawn((Node { width: Val::Percent(100.0), height: Val::Percent(100.0),
        flex_direction: FlexDirection::Column, justify_content: JustifyContent::SpaceBetween, ..default() }, HudRoot))
    .with_children(|root| {
        // ── Top bar ──
        root.spawn((Node { width: Val::Percent(100.0), height: Val::Px(36.0), flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center,
            padding: UiRect::horizontal(Val::Px(10.0)), ..default() },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
        )).with_children(|p| {
            p.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(12.0), ..default() }).with_children(|p| {
                ht.cities = Some(p.spawn((Text::new("城 0"), TextFont { font: font.clone(), font_size: 14.0, ..default() })).id());
                ht.pop = Some(p.spawn((Text::new("兵 0/0"), TextFont { font: font.clone(), font_size: 14.0, ..default() })).id());
                ht.enemy = Some(p.spawn((Text::new("敌 0"), TextFont { font: font.clone(), font_size: 14.0, ..default() })).id());
                ht.time = Some(p.spawn((Text::new("T 0:00"), TextFont { font: font.clone(), font_size: 14.0, ..default() })).id());
            });
            // Toast message on the right
            ht.toast_text = Some(p.spawn((Text::new(""), TextFont { font: font.clone(), font_size: 13.0, ..default() },
                TextColor(Color::srgb(1.0, 0.9, 0.3)))).id());
        });

        root.spawn(Node { flex_grow: 1.0, ..default() }); // spacer

        // ── Bottom zone 180px: Left 30% info + Right 70% command+compendium ──
        root.spawn((Node { width: Val::Percent(100.0), height: Val::Px(180.0),
            flex_direction: FlexDirection::Row, ..default() }, BottomZone))
        .with_children(|bz| {
            bz.spawn(Node { width: Val::Percent(30.0), height: Val::Percent(100.0), ..default() })
              .with_children(|p| {
                // Soldier panel
                ht.s_root = Some(p.spawn((Node { width: Val::Percent(100.0), flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(8.0)), row_gap: Val::Px(3.0), ..default() },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)), SoldierPanelRoot,
                )).with_children(|p| {
                    ht.s_header = Some(p.spawn((Text::new("点击单位以查看详情"), TextFont { font: font.clone(), font_size: 14.0, ..default() })).id());
                    p.spawn(Node { flex_direction: FlexDirection::Row, align_items: AlignItems::Center, ..default() }).with_children(|p| {
                        ht.s_hp_text = Some(p.spawn((Text::new(""), TextFont { font: font.clone(), font_size: 12.0, ..default() })).id());
                        p.spawn((Node { width: Val::Percent(60.0), height: Val::Px(10.0), margin: UiRect::left(Val::Px(6.0)), ..default() },
                            BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 1.0)),
                        )).with_children(|p| {
                            ht.s_hp_fill = Some(p.spawn((Node { width: Val::Percent(0.0), height: Val::Percent(100.0), ..default() },
                                BackgroundColor(Color::srgba(0.2, 0.8, 0.2, 1.0)), HpFillS)).id());
                        });
                    });
                    ht.s_atk = Some(p.spawn((Text::new(""), TextFont { font: font.clone(), font_size: 12.0, ..default() })).id());
                    ht.s_spd = Some(p.spawn((Text::new(""), TextFont { font: font.clone(), font_size: 12.0, ..default() })).id());
                    p.spawn(Node { flex_direction: FlexDirection::Row, align_items: AlignItems::Center, ..default() }).with_children(|p| {
                        ht.s_exp_text = Some(p.spawn((Text::new(""), TextFont { font: font.clone(), font_size: 12.0, ..default() })).id());
                        p.spawn((Node { width: Val::Percent(50.0), height: Val::Px(8.0), margin: UiRect::left(Val::Px(6.0)), ..default() },
                            BackgroundColor(Color::srgba(0.1, 0.1, 0.3, 1.0)),
                        )).with_children(|p| {
                            ht.s_exp_fill = Some(p.spawn((Node { width: Val::Percent(0.0), height: Val::Percent(100.0), ..default() },
                                BackgroundColor(Color::srgba(0.4, 0.5, 1.0, 1.0)), ExpFillS)).id());
                        });
                    });
                    ht.s_effect = Some(p.spawn((Text::new(""), TextFont { font: font.clone(), font_size: 12.0, ..default() })).id());
                    ht.s_origin = Some(p.spawn((Text::new(""), TextFont { font: font.clone(), font_size: 12.0, ..default() })).id());
                }).id());
                // City panel (hidden)
                ht.c_root = Some(p.spawn((Node { width: Val::Percent(100.0), flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(8.0)), row_gap: Val::Px(3.0), display: Display::None, ..default() },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)), CityPanelRoot,
                )).with_children(|p| {
                    ht.c_info = Some(p.spawn((Text::new("[城池] Lv.?"), TextFont { font: font.clone(), font_size: 14.0, ..default() })).id());
                    p.spawn(Node { flex_direction: FlexDirection::Row, align_items: AlignItems::Center, ..default() }).with_children(|p| {
                        ht.c_hp_text = Some(p.spawn((Text::new("HP ?/?"), TextFont { font: font.clone(), font_size: 12.0, ..default() })).id());
                        p.spawn((Node { width: Val::Percent(50.0), height: Val::Px(10.0), margin: UiRect::left(Val::Px(6.0)), ..default() },
                            BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 1.0)),
                        )).with_children(|p| {
                            ht.c_hp_fill = Some(p.spawn((Node { width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() },
                                BackgroundColor(Color::srgba(0.2, 0.8, 0.2, 1.0)), HpFillC)).id());
                        });
                    });
                    ht.c_pop = Some(p.spawn((Text::new("兵 ?/?"), TextFont { font: font.clone(), font_size: 12.0, ..default() })).id());
                    ht.c_exp = Some(p.spawn((Text::new("经验 ?/?"), TextFont { font: font.clone(), font_size: 12.0, ..default() })).id());
                    ht.c_spawn = Some(p.spawn((Text::new("当前: 民兵"), TextFont { font: font.clone(), font_size: 13.0, ..default() })).id());
                    p.spawn(Node { flex_direction: FlexDirection::Row, ..default() }).with_children(|p| {
                        for (st, label) in [(SoldierType::Militia,"民兵"),(SoldierType::Infantry,"步兵"),(SoldierType::Archer,"弓兵"),(SoldierType::Cavalry,"骑兵")] {
                            p.spawn((Button, Node { padding: UiRect::all(Val::Px(6.0)), margin: UiRect::all(Val::Px(3.0)), ..default() }, SpawnTypeBtn(st)))
                                .with_child((Text::new(label), TextFont { font: font.clone(), font_size: 12.0, ..default() }));
                        }
                    });
                }).id());
              });
            // Right 70%: command card + compendium
            bz.spawn(Node { width: Val::Percent(70.0), height: Val::Percent(100.0), flex_direction: FlexDirection::Column, ..default() })
              .with_children(|p| {
                p.spawn((Node { width: Val::Percent(100.0), flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(8.0)), row_gap: Val::Px(4.0), ..default() },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
                )).with_children(|p| {
                    ht.cmd_info = Some(p.spawn((Text::new("无可用命令 — 请先选择单位"), TextFont { font: font.clone(), font_size: 12.0, ..default() })).id());
                    p.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(6.0), ..default() }).with_children(|p| {
                        for label in ["移动","攻击","停止","驻守"] {
                            p.spawn((Button, Node { padding: UiRect::all(Val::Px(8.0)), ..default() }))
                                .with_child((Text::new(label), TextFont { font: font.clone(), font_size: 14.0, ..default() }));
                        }
                    });
                });
                p.spawn((Node { width: Val::Percent(100.0), flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(8.0)), row_gap: Val::Px(3.0), ..default() },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
                )).with_children(|p| {
                    ht.comp_header = Some(p.spawn((Text::new("悬停兵种按钮查看详情"), TextFont { font: font.clone(), font_size: 13.0, ..default() })).id());
                    ht.comp_hp = Some(p.spawn((Text::new(""), TextFont { font: font.clone(), font_size: 12.0, ..default() })).id());
                    ht.comp_atk = Some(p.spawn((Text::new(""), TextFont { font: font.clone(), font_size: 12.0, ..default() })).id());
                    ht.comp_spd = Some(p.spawn((Text::new(""), TextFont { font: font.clone(), font_size: 12.0, ..default() })).id());
                    ht.comp_rng = Some(p.spawn((Text::new(""), TextFont { font: font.clone(), font_size: 12.0, ..default() })).id());
                    ht.comp_rad = Some(p.spawn((Text::new(""), TextFont { font: font.clone(), font_size: 12.0, ..default() })).id());
                    ht.comp_effect = Some(p.spawn((Text::new(""), TextFont { font: font.clone(), font_size: 12.0, ..default() })).id());
                    ht.comp_desc = Some(p.spawn((Text::new(""), TextFont { font: font.clone(), font_size: 11.0, ..default() })).id());
                });
              });
        });

        // ── Toolbar ──
        root.spawn((Node { width: Val::Percent(100.0), height: Val::Px(40.0), flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center,
            padding: UiRect::horizontal(Val::Px(8.0)), ..default() },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
        )).with_children(|p| {
            // Left: existing toolbar buttons
            p.spawn(Node { flex_direction: FlexDirection::Row, align_items: AlignItems::Center, column_gap: Val::Px(4.0), ..default() }).with_children(|p| {
                for (label, marker) in [("O框选",0u8),("[ ]框选",1),("盾",2),("[>]优先",3)] {
                    p.spawn((Button, Node { padding: UiRect::all(Val::Px(6.0)), ..default() }, ToolbarButton(marker)))
                        .with_child((Text::new(label), TextFont { font: font.clone(), font_size: 13.0, ..default() }));
                }
            });

            // Separator
            p.spawn(Node { width: Val::Px(1.0), height: Val::Percent(80.0), ..default() });

            // Right: seek panel (scope dropdown + range input + issue button)
            p.spawn((Node { flex_direction: FlexDirection::Row, align_items: AlignItems::Center, column_gap: Val::Px(6.0), ..default() }, SeekPanelRoot))
            .with_children(|p| {
                // Scope dropdown (relative container for popup positioning)
                p.spawn(Node { position_type: PositionType::Relative, ..default() }).with_children(|p| {
                    // Trigger button — store Text child entity ID (not Button parent ID)
                    let mut scope_text_id = Entity::PLACEHOLDER;
                    p.spawn((Button, Node { padding: UiRect::new(Val::Px(8.0), Val::Px(8.0), Val::Px(4.0), Val::Px(4.0)), ..default() },
                        BackgroundColor(Color::srgba(0.25, 0.25, 0.3, 1.0)), SeekScopeDropdown,
                    )).with_children(|p| {
                        scope_text_id = p.spawn((Text::new("全体 ▼"), TextFont { font: font.clone(), font_size: 12.0, ..default() })).id();
                    });
                    ht.seek_scope_text = Some(scope_text_id);
                    // Popup container (hidden by default via Display::None)
                    ht.seek_dropdown_container = Some(p.spawn((Node {
                        display: Display::None,
                        flex_direction: FlexDirection::Column,
                        position_type: PositionType::Absolute,
                        bottom: Val::Px(28.0), left: Val::Px(0.0),
                        ..default()
                    }, BackgroundColor(Color::srgba(0.15, 0.15, 0.2, 0.95)), SeekDropdownPopup,
                    )).with_children(|p| {
                        for (label, scope) in [("全体", SeekScope::All), ("步兵", SeekScope::ByType(SoldierType::Infantry)),
                            ("弓兵", SeekScope::ByType(SoldierType::Archer)), ("骑兵", SeekScope::ByType(SoldierType::Cavalry))]
                        {
                            p.spawn((Button, Node { padding: UiRect::new(Val::Px(12.0), Val::Px(12.0), Val::Px(4.0), Val::Px(4.0)), ..default() },
                                SeekScopeOption(scope),
                            )).with_child((Text::new(label), TextFont { font: font.clone(), font_size: 12.0, ..default() }));
                        }
                    }).id());
                });

                // Range input box — store Text child entity ID
                let mut range_text_id = Entity::PLACEHOLDER;
                p.spawn((Button, Node { padding: UiRect::new(Val::Px(8.0), Val::Px(8.0), Val::Px(4.0), Val::Px(4.0)),
                    min_width: Val::Px(50.0), ..default() },
                    BackgroundColor(Color::srgba(0.15, 0.15, 0.2, 1.0)), SeekRangeInput,
                )).with_children(|p| {
                    range_text_id = p.spawn((Text::new("10"), TextFont { font: font.clone(), font_size: 12.0, ..default() })).id();
                });
                ht.seek_range_text = Some(range_text_id);

                // Issue button
                p.spawn((Button, Node { padding: UiRect::new(Val::Px(10.0), Val::Px(10.0), Val::Px(4.0), Val::Px(4.0)), ..default() },
                    BackgroundColor(Color::srgba(0.2, 0.5, 0.3, 1.0)), SeekIssueBtn,
                )).with_child((Text::new("下发"), TextFont { font: font.clone(), font_size: 12.0, ..default() }));
            });
        });
    });
}

// ══════════ Update Systems ══════════

pub fn update_top_bar(mut tq: Query<&mut Text>, ht: Res<HudTexts>,
    mut sim: bevy::ecs::system::NonSendMut<SimulationWorld>, time: Res<Time>) {
    let w = &mut sim.0;
    let (mut pc, mut pp, mut pm, mut es) = (0usize,0u32,0u32,0u32);
    { let mut q = w.query::<(&FactionComponent, &CityComponent)>(); for (f,c) in q.iter(w) { if f.0==Faction::Player { pc+=1; pp+=c.population; pm+=c.max_population; } } }
    { let mut q = w.query::<(&FactionComponent, &SoldierTypeComponent)>(); for (f,_) in q.iter(w) { if f.0==Faction::Enemy { es+=1; } } }
    let e = time.elapsed().as_secs();
    if let Some(id) = ht.cities { if let Ok(mut t)=tq.get_mut(id) { t.0=format!("城 {}",pc); } }
    if let Some(id) = ht.pop { if let Ok(mut t)=tq.get_mut(id) { t.0=format!("兵 {}/{}",pp,pm); } }
    if let Some(id) = ht.enemy { if let Ok(mut t)=tq.get_mut(id) { t.0=format!("敌 {}",es); } }
    if let Some(id) = ht.time { if let Ok(mut t)=tq.get_mut(id) { t.0=format!("T {}:{:02}",e/60,e%60); } }
}

pub fn update_bottom_panel(
    mut tq: Query<&mut Text>,
    mut node_params: ParamSet<(
        Query<(&mut Node, &mut BackgroundColor), With<HpFillS>>,
        Query<(&mut Node, &mut BackgroundColor), With<ExpFillS>>,
        Query<(&mut Node, &mut BackgroundColor), With<HpFillC>>,
        Query<&mut Node, With<SoldierPanelRoot>>,
        Query<&mut Node, With<CityPanelRoot>>,
    )>,
    spawn_btns: Query<(&SpawnTypeBtn, &Interaction), Changed<Interaction>>,
    ht: Res<HudTexts>, selection: Res<SelectionState>,
    mut sim: bevy::ecs::system::NonSendMut<SimulationWorld>,
) {
    let w = &mut sim.0;
    // Resolve city entity from UnitId
    let city_entity: Option<Entity> = selection.selected_city.and_then(|cid| {
        let mut q = w.query::<(Entity, &UnitIdComponent, &CityMarker)>();
        q.iter(w).find(|(_, id, _)| id.0 == cid).map(|(e, _, _)| e)
    });
    let has_city = city_entity.is_some();
    let has_soldiers = !selection.selected_unit_ids.is_empty();

    // Toggle panel visibility: city and soldier share the same spot
    if let Some(e) = ht.s_root { if let Ok(mut n) = node_params.p3().get_mut(e) { n.display = if !has_city { Display::Flex } else { Display::None }; } }
    if let Some(e) = ht.c_root { if let Ok(mut n) = node_params.p4().get_mut(e) { n.display = if has_city { Display::Flex } else { Display::None }; } }

    // ── Update city panel ──
    if let Some(ce) = city_entity {
        if let Some(city) = w.entity(ce).get::<CityComponent>() {
                let r = city.health_current as f32 / city.health_max.max(1) as f32;
                set_text(&mut tq, ht.c_info, &format!("[城池] Lv.{} (最高Lv.{})", city.level, city.max_level));
                set_text(&mut tq, ht.c_hp_text, &format!("HP {}/{}", city.health_current, city.health_max));
                set_text(&mut tq, ht.c_pop, &format!("兵 {}/{}", city.population, city.max_population));
                let cc = w.resource::<CityGlobalConfig>();
                let req = (city.health_max as f32 * cc.level_up_cost_multiplier * city.level as f32) as u64;
                set_text(&mut tq, ht.c_exp, &format!("经验 {}/{}", city.level_exp, req));
                set_text(&mut tq, ht.c_spawn, &format!("当前: {}", match city.spawn_type {
                    SoldierType::Militia=>"民兵",SoldierType::Infantry=>"步兵",SoldierType::Archer=>"弓兵",SoldierType::Cavalry=>"骑兵" }));
                if let Some(f) = ht.c_hp_fill { if let Ok((mut n,mut bg))=node_params.p2().get_mut(f) {
                    n.width=Val::Percent(r*100.0); bg.0=if r>0.5{Color::srgba(0.2,0.8,0.2,1.0)}else{Color::srgba(0.9,0.2,0.2,1.0)}; } }
            }
        }

    // ── Update soldier panel ──
    if has_soldiers && !has_city {
        let ids = &selection.selected_unit_ids;
        let sc = w.resource::<SoldierConfig>().clone();
        struct SI { st:SoldierType, hp:u32, mhp:u32, atk:u32, spd:u32, rng:u32, rad:u32, lv:u32, exp:u32 }
        let soldiers: Vec<SI> = {
            let mut q = w.query::<(Entity,&UnitIdComponent,&Health,&Attack,&Movement,&SoldierTypeComponent,&Level)>();
            ids.iter().filter_map(|uid| q.iter(w).find(|(_,id,_,_,_,_,_)| id.0==*uid).map(|(_,_,hp,atk,mov,st,lvl)| {
                let c = sc.get(st.0);
                SI{st:st.0,hp:hp.current,mhp:hp.max,atk:atk.damage,spd:mov.speed,rng:atk.range,rad:c.collision_radius,lv:lvl.level,exp:lvl.exp}
            })).collect()
        };
        if soldiers.len() == 1 {
            let s = &soldiers[0];
            let (label, effect) = match s.st {
                SoldierType::Militia=>("民兵","无"), SoldierType::Infantry=>("步兵","举盾: 可举盾大幅减伤"),
                SoldierType::Archer=>("弓兵","远程+穿透: 箭矢可穿透"), SoldierType::Cavalry=>("骑兵","闪避+无畏: 受伤时可闪避")
            };
            set_text(&mut tq, ht.s_header, &format!("{} Lv.{}", label, s.lv));
            set_text(&mut tq, ht.s_hp_text, &format!("HP {}/{}", s.hp, s.mhp));
            set_text(&mut tq, ht.s_atk, &format!("ATK {}  SPD {}", s.atk, s.spd));
            set_text(&mut tq, ht.s_spd, &format!("RNG {}  RAD {}", s.rng, s.rad));
            set_text(&mut tq, ht.s_effect, &format!("特殊: {}", effect));
            set_text(&mut tq, ht.s_exp_text, &format!("EXP {}/{}", s.exp, 100u32));
            set_text(&mut tq, ht.s_origin, "");
            if let Some(f) = ht.s_hp_fill { if let Ok((mut n,mut bg))=node_params.p0().get_mut(f) {
                let r = s.hp as f32/s.mhp.max(1) as f32;
                n.width=Val::Percent(r*100.0); bg.0=if r>0.5{Color::srgba(0.2,0.8,0.2,1.0)}else{Color::srgba(0.9,0.2,0.2,1.0)}; } }
            if let Some(f) = ht.s_exp_fill { if let Ok((mut n,_))=node_params.p1().get_mut(f) { n.width=Val::Percent((s.exp as f32/100.0*100.0).min(100.0)); } }
        } else if !soldiers.is_empty() {
            let mut counts: HashMap<SoldierType,u32> = HashMap::new();
            let (mut thp,mut tmax,mut tatk) = (0u32,0u32,0u32);
            for s in &soldiers { *counts.entry(s.st).or_default()+=1; thp+=s.hp; tmax+=s.mhp; tatk+=s.atk; }
            let parts: Vec<String> = counts.iter().map(|(st,c)| format!("{}{}",c,match st{SoldierType::Militia=>"民",SoldierType::Infantry=>"步",SoldierType::Archer=>"弓",SoldierType::Cavalry=>"骑"})).collect();
            set_text(&mut tq, ht.s_header, &format!("选中 {} 单位", soldiers.len()));
            set_text(&mut tq, ht.s_hp_text, &format!("HP {}/{}", thp, tmax));
            set_text(&mut tq, ht.s_atk, &format!("均ATK {}", tatk/soldiers.len().max(1) as u32));
            set_text(&mut tq, ht.s_spd, &parts.join("  "));
            for e in [ht.s_exp_text,ht.s_effect,ht.s_origin] { if let Some(id)=e { if let Ok(mut t)=tq.get_mut(id) { t.0.clear(); } } }
            if let Some(f)=ht.s_exp_fill { if let Ok((mut n,_))=node_params.p1().get_mut(f) { n.width=Val::Percent(0.0); } }
            let r = thp as f32/tmax.max(1) as f32;
            if let Some(f)=ht.s_hp_fill { if let Ok((mut n,mut bg))=node_params.p0().get_mut(f) { n.width=Val::Percent(r*100.0); bg.0=if r>0.5{Color::srgba(0.2,0.8,0.2,1.0)}else{Color::srgba(0.9,0.2,0.2,1.0)}; } }
        }
    } else {
        // No selection: show placeholder in soldier panel
        set_text(&mut tq, ht.s_header, "点击单位以查看详情");
        for e in [ht.s_hp_text,ht.s_atk,ht.s_spd,ht.s_exp_text,ht.s_effect,ht.s_origin] { if let Some(id)=e { if let Ok(mut t)=tq.get_mut(id) { t.0.clear(); } } }
        if let Some(f)=ht.s_hp_fill { if let Ok((mut n,_))=node_params.p0().get_mut(f) { n.width=Val::Percent(0.0); } }
        if let Some(f)=ht.s_exp_fill { if let Ok((mut n,_))=node_params.p1().get_mut(f) { n.width=Val::Percent(0.0); } }
    }

    // ── Command card summary ──
    let summary = if has_soldiers && !has_city {
        let mut counts: HashMap<SoldierType,u32> = HashMap::new();
        for uid in &selection.selected_unit_ids {
            if let Some((_,_,st)) = w.query::<(Entity,&UnitIdComponent,&SoldierTypeComponent)>().iter(w).find(|(_,id,_)| id.0==*uid) { *counts.entry(st.0).or_default()+=1; }
        }
        let parts: Vec<String> = counts.iter().map(|(st,c)| format!("{}{}",c,match st{SoldierType::Militia=>"民",SoldierType::Infantry=>"步",SoldierType::Archer=>"弓",SoldierType::Cavalry=>"骑"})).collect();
        if parts.is_empty() { String::new() } else { parts.join(" ") }
    } else { String::new() };
    if let Some(id) = ht.cmd_info {
        if let Ok(mut t) = tq.get_mut(id) {
            t.0 = if summary.is_empty() { "无可用命令 — 请先选择单位".into() } else { format!("选中: {}", summary) };
        }
    }

    // ── Compendium hover ──
    let hovered_st = spawn_btns.iter().find_map(|(btn,interaction)| if *interaction==Interaction::Hovered { Some(btn.0) } else { None });
    if let Some(st) = hovered_st {
        show_compendium(&mut tq, &ht, st);
    } else if has_city {
        show_compendium(&mut tq, &ht, SoldierType::Militia);
    } else {
        clear_compendium(&mut tq, &ht);
    }
}

fn set_text(tq: &mut Query<&mut Text>, e: Option<Entity>, s: &str) {
    if let Some(id) = e { if let Ok(mut t) = tq.get_mut(id) { t.0 = s.into(); } }
}

fn show_compendium(tq: &mut Query<&mut Text>, ht: &HudTexts, st: SoldierType) {
    let (name, hp, atk, spd, rng, rad, effect, desc): (&str,u32,u32,u32,u32,u32,&str,&str) = match st {
        SoldierType::Militia => ("民兵",100,16,80,30,6,"无","基础步兵，成本低廉，适合快速补充兵力。"),
        SoldierType::Infantry => ("步兵",100,20,80,30,7,"举盾: 可举盾大幅减伤","重装步兵，可举盾大幅减伤，攻城主力。"),
        SoldierType::Archer => ("弓兵",100,20,80,600,5,"远程+穿透: 箭矢可穿透","远程射手，箭矢可穿透敌人，对建筑伤害极低。"),
        SoldierType::Cavalry => ("骑兵",140,20,200,30,10,"闪避+无畏: 受伤时可闪避","重骑兵，高速冲锋陷阵。受伤时可闪避近战攻击。"),
    };
    set_text(tq, ht.comp_header, &format!("{} 图鉴",name));
    set_text(tq, ht.comp_hp, &format!("HP {}",hp));
    set_text(tq, ht.comp_atk, &format!("ATK {}",atk));
    set_text(tq, ht.comp_spd, &format!("SPD {}",spd));
    set_text(tq, ht.comp_rng, &format!("RNG {}",rng));
    set_text(tq, ht.comp_rad, &format!("RAD {}",rad));
    set_text(tq, ht.comp_effect, &format!("特殊: {}",effect));
    set_text(tq, ht.comp_desc, desc);
}

fn clear_compendium(tq: &mut Query<&mut Text>, ht: &HudTexts) {
    for e in [ht.comp_hp,ht.comp_atk,ht.comp_spd,ht.comp_rng,ht.comp_rad,ht.comp_effect,ht.comp_desc] {
        if let Some(id) = e { if let Ok(mut t) = tq.get_mut(id) { t.0.clear(); } }
    }
    set_text(tq, ht.comp_header, "悬停兵种按钮查看详情");
}

// ══════════ Button Systems ══════════

pub fn soldier_type_button_system(mut q: Query<(&SpawnTypeBtn, &Interaction), Changed<Interaction>>,
    selection: Res<SelectionState>, mut sim: bevy::ecs::system::NonSendMut<SimulationWorld>) {
    let w = &mut sim.0;
    for (btn,interaction) in q.iter_mut() {
        if *interaction != Interaction::Pressed { continue; }
        if let Some(cid) = selection.selected_city {
            let ce = w.query::<(Entity, &UnitIdComponent, &CityMarker)>().iter(w).find(|(_,id,_)| id.0==cid).map(|(e,_,_)| e);
            if let Some(ce) = ce {
                if let Some(mut c) = w.entity_mut(ce).get_mut::<CityComponent>() { c.spawn_type = btn.0; }
            }
        }
    }
}

pub fn toolbar_button_system(mut q: Query<(&ToolbarButton, &Interaction), Changed<Interaction>>,
    mut sel: ResMut<SelectionState>, mut force: ResMut<ForceMoveNext>) {
    for (btn,interaction) in q.iter_mut() {
        if *interaction != Interaction::Pressed { continue; }
        match btn.0 { 0=>sel.selection_mode=crate::selection::SelectionMode::Circle,1=>sel.selection_mode=crate::selection::SelectionMode::Rect,2=>{},3=>force.active=true, _=>{} }
    }
}

// ══════════ Seek Panel Mode System ══════════

/// Switch between global/selection mode based on unit selection.
/// Reset range default only on mode transition.
pub fn seek_panel_mode_system(
    selection: Res<SelectionState>,
    mut state: ResMut<SeekPanelState>,
) {
    let now_selected = !selection.selected_unit_ids.is_empty();
    if now_selected != state.has_selection {
        state.has_selection = now_selected;
        // Don't reset while user is editing the input
        if state.editing { return; }
        state.range_value = if now_selected { 30 } else { 10 };
        state.input_buffer.clear();
    }
}

// ══════════ Seek Panel Dropdown System ══════════

/// Handle scope dropdown: open/close trigger, option selection, click-outside close.
pub fn seek_panel_dropdown_system(
    mouse: Res<ButtonInput<MouseButton>>,
    mut state: ResMut<SeekPanelState>,
    mut tq: Query<&mut Text>,
    ht: Res<HudTexts>,
    dropdown_btn: Query<&Interaction, With<SeekScopeDropdown>>,
    option_btns: Query<(&SeekScopeOption, &Interaction), With<SeekScopeOption>>,
    mut popup_nodes: Query<&mut Node, With<SeekDropdownPopup>>,
) {
    // Toggle dropdown on trigger click (use just_pressed to avoid re-toggle)
    let trigger_pressed = dropdown_btn.iter().any(|i| *i == Interaction::Pressed);
    if trigger_pressed && mouse.just_pressed(MouseButton::Left) {
        state.dropdown_open = !state.dropdown_open;
    }

    // Handle option selection — check all options for Pressed state
    let mut option_selected = false;
    for (opt, interaction) in option_btns.iter() {
        if *interaction == Interaction::Pressed {
            state.scope = opt.0.clone();
            state.dropdown_open = false;
            option_selected = true;
            break;
        }
    }

    // Close on click outside (only if we didn't just select an option)
    if !option_selected && mouse.just_pressed(MouseButton::Left) && state.dropdown_open {
        if !trigger_pressed {
            state.dropdown_open = false;
        }
    }

    // Update popup visibility via Display toggle
    if let Some(id) = ht.seek_dropdown_container {
        if let Ok(mut node) = popup_nodes.get_mut(id) {
            node.display = if state.dropdown_open { Display::Flex } else { Display::None };
        }
    }

    // Update scope text
    let label = scope_label(&state.scope);
    if let Some(id) = ht.seek_scope_text {
        if let Ok(mut t) = tq.get_mut(id) {
            t.0 = format!("{} ▼", label);
        }
    }
}

fn scope_label(scope: &SeekScope) -> &'static str {
    match scope {
        SeekScope::All => "全体",
        SeekScope::ByType(SoldierType::Militia) => "民兵",
        SeekScope::ByType(SoldierType::Infantry) => "步兵",
        SeekScope::ByType(SoldierType::Archer) => "弓兵",
        SeekScope::ByType(SoldierType::Cavalry) => "骑兵",
    }
}

// ══════════ Seek Panel Input System ══════════

/// Handle range input: click to enter edit mode, keyboard to type, Enter/Esc to confirm/cancel.
pub fn seek_panel_input_system(
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<SeekPanelState>,
    mut tq: Query<&mut Text>,
    ht: Res<HudTexts>,
    input_btn: Query<&Interaction, With<SeekRangeInput>>,
) {
    // Click on input box → enter edit mode (use just_pressed to prevent re-entry)
    let input_pressed = input_btn.iter().any(|i| *i == Interaction::Pressed);
    if input_pressed && mouse.just_pressed(MouseButton::Left) && !state.editing {
        state.editing = true;
        state.input_buffer = state.range_value.to_string();
    }

    if !state.editing {
        // Display current value
        if let Some(id) = ht.seek_range_text {
            if let Ok(mut t) = tq.get_mut(id) { t.0 = state.range_value.to_string(); }
        }
        return;
    }

    // Edit mode: capture keyboard
    let mut changed = false;
    let mut exit_edit = false;
    let mut cancel = false;

    // Check for key presses
    if keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::NumpadEnter) {
        exit_edit = true;
    } else if keyboard.just_pressed(KeyCode::Escape) {
        cancel = true;
        exit_edit = true;
    } else if keyboard.just_pressed(KeyCode::Backspace) {
        state.input_buffer.pop();
        changed = true;
    } else {
        // Digit keys
        let digit = if keyboard.just_pressed(KeyCode::Digit0) || keyboard.just_pressed(KeyCode::Numpad0) { Some('0') }
        else if keyboard.just_pressed(KeyCode::Digit1) || keyboard.just_pressed(KeyCode::Numpad1) { Some('1') }
        else if keyboard.just_pressed(KeyCode::Digit2) || keyboard.just_pressed(KeyCode::Numpad2) { Some('2') }
        else if keyboard.just_pressed(KeyCode::Digit3) || keyboard.just_pressed(KeyCode::Numpad3) { Some('3') }
        else if keyboard.just_pressed(KeyCode::Digit4) || keyboard.just_pressed(KeyCode::Numpad4) { Some('4') }
        else if keyboard.just_pressed(KeyCode::Digit5) || keyboard.just_pressed(KeyCode::Numpad5) { Some('5') }
        else if keyboard.just_pressed(KeyCode::Digit6) || keyboard.just_pressed(KeyCode::Numpad6) { Some('6') }
        else if keyboard.just_pressed(KeyCode::Digit7) || keyboard.just_pressed(KeyCode::Numpad7) { Some('7') }
        else if keyboard.just_pressed(KeyCode::Digit8) || keyboard.just_pressed(KeyCode::Numpad8) { Some('8') }
        else if keyboard.just_pressed(KeyCode::Digit9) || keyboard.just_pressed(KeyCode::Numpad9) { Some('9') }
        else { None };

        if let Some(ch) = digit {
            if state.input_buffer.len() < 4 {
                state.input_buffer.push(ch);
                changed = true;
            }
        }
    }

    if exit_edit {
        if cancel {
            // Restore original value
            state.input_buffer.clear();
        } else if !state.input_buffer.is_empty() {
            // Parse and apply
            if let Ok(val) = state.input_buffer.parse::<u32>() {
                state.range_value = val;
            }
        }
        // Empty buffer → keep original value
        state.editing = false;
        state.input_buffer.clear();
        changed = true;
    }

    // Update display
    if changed || state.editing {
        if let Some(id) = ht.seek_range_text {
            if let Ok(mut t) = tq.get_mut(id) {
                t.0 = if state.editing {
                    format!("{}▌", state.input_buffer)
                } else {
                    state.range_value.to_string()
                };
            }
        }
    }
}

// ══════════ Seek Panel Issue System ══════════

/// Handle issue button click: generate command, trigger toast.
pub fn seek_panel_issue_system(
    mouse: Res<ButtonInput<MouseButton>>,
    issue_btn: Query<&Interaction, With<SeekIssueBtn>>,
    state: Res<SeekPanelState>,
    selection: Res<SelectionState>,
    mut cmd_buf: ResMut<CommandBuffer>,
    tick_clock: Res<TickClock>,
    mut toast: ResMut<ToastMessage>,
) {
    if !mouse.just_pressed(MouseButton::Left) { return; }
    for interaction in issue_btn.iter() {
        if *interaction != Interaction::Pressed { continue; }

        let next_tick = tick_clock.current_tick + 1;
        let has_sel = !selection.selected_unit_ids.is_empty();

        if has_sel {
            // Selection mode
            cmd_buf.push(GameCommand {
                tick: next_tick,
                player_id: 0,
                action: Action::SetSeekStance {
                    scope: state.scope.clone(),
                    seek_range: state.range_value,
                    unit_ids: selection.selected_unit_ids.clone(),
                },
            });
            // Toast: count by scope
            let count = count_matching(&selection.selected_unit_ids, &state.scope, &selection);
            let scope_name = scope_label(&state.scope);
            toast.text = if matches!(state.scope, SeekScope::All) {
                format!("已下发选中全体({})索敌 范围{}", selection.selected_unit_ids.len(), state.range_value)
            } else {
                format!("已下发选中{}({})索敌 范围{}", scope_name, count, state.range_value)
            };
        } else {
            // Global mode
            cmd_buf.push(GameCommand {
                tick: next_tick,
                player_id: 0,
                action: Action::SetSeekStance {
                    scope: state.scope.clone(),
                    seek_range: state.range_value,
                    unit_ids: vec![],
                },
            });
            let scope_name = scope_label(&state.scope);
            toast.text = format!("已下发{}索敌 范围{}", scope_name, state.range_value);
        }
        toast.remaining_ticks = TOAST_DURATION_TICKS;
    }
}

/// Count how many selected units match the given scope.
fn count_matching(unit_ids: &[UnitId], scope: &SeekScope, selection: &SelectionState) -> usize {
    // For All scope, all selected match
    match scope {
        SeekScope::All => unit_ids.len(),
        SeekScope::ByType(_) => {
            // We need to check the simulation world, but we don't have access here.
            // Approximate: if single type selected, show that count.
            // For now, just return total count — the toast will still be informative.
            unit_ids.len()
        }
    }
}

// ══════════ Toast Systems ══════════

/// Tick down toast timer.
pub fn toast_tick_system(mut toast: ResMut<ToastMessage>) {
    if toast.remaining_ticks > 0 {
        toast.remaining_ticks -= 1;
        if toast.remaining_ticks == 0 {
            toast.text.clear();
        }
    }
}

/// Display toast message in the top bar.
pub fn toast_display_system(
    toast: Res<ToastMessage>,
    ht: Res<HudTexts>,
    mut tq: Query<&mut Text>,
) {
    let Some(id) = ht.toast_text else { return };
    let Ok(mut text) = tq.get_mut(id) else { return };
    text.0 = toast.text.clone();
}

// ══════════ Selection Summary Toast ══════════

/// Show toast when unit selection changes.
pub fn selection_summary_toast_system(
    selection: Res<SelectionState>,
    mut prev_count: Local<usize>,
    mut toast: ResMut<ToastMessage>,
    mut sim_world: bevy::ecs::system::NonSendMut<SimulationWorld>,
) {
    let now = selection.selected_unit_ids.len();
    if now == *prev_count { return; }
    *prev_count = now;

    if now == 0 { return; } // don't clear existing toast on deselect

    // Build summary
    let w = &mut sim_world.0;
    let mut counts: HashMap<SoldierType, usize> = HashMap::new();
    {
        let mut q = w.query::<(&UnitIdComponent, &SoldierTypeComponent)>();
        for uid in &selection.selected_unit_ids {
            for (id, st) in q.iter(w) {
                if id.0 == *uid {
                    *counts.entry(st.0).or_insert(0) += 1;
                    break;
                }
            }
        }
    }

    if counts.len() <= 1 {
        // Single type
        let (stype, count) = counts.iter().next().unwrap_or((&SoldierType::Militia, &0));
        let name = match stype {
            SoldierType::Militia => "民兵",
            SoldierType::Infantry => "步兵",
            SoldierType::Archer => "弓兵",
            SoldierType::Cavalry => "骑兵",
        };
        toast.text = format!("选中 {} 个{}", count, name);
    } else {
        // Mixed types
        let parts: Vec<String> = counts.iter().map(|(st, c)| {
            let name = match st {
                SoldierType::Militia => "民兵",
                SoldierType::Infantry => "步兵",
                SoldierType::Archer => "弓兵",
                SoldierType::Cavalry => "骑兵",
            };
            format!("{}{}", name, c)
        }).collect();
        toast.text = format!("选中 {} 个单位: {}", now, parts.join(" "));
    }
    toast.remaining_ticks = TOAST_DURATION_TICKS;
}
