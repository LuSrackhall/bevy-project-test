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

#[derive(Resource, Default)] pub struct SelectedCity(pub Option<Entity>);

#[derive(Resource, Default)]
pub(crate) struct HudTexts {
    // top bar
    cities: Option<Entity>, pop: Option<Entity>, enemy: Option<Entity>, time: Option<Entity>,
    // soldier panel
    s_header: Option<Entity>, s_hp_text: Option<Entity>, s_hp_fill: Option<Entity>,
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
}

// ══════════ Marker Components ══════════

#[derive(Component)] struct HudRoot;
#[derive(Component)] struct BottomZone;
#[derive(Component)] pub(crate) struct HpFillS;
#[derive(Component)] pub(crate) struct ExpFillS;
#[derive(Component)] pub(crate) struct HpFillC;
#[derive(Component)] pub(crate) struct CityPanelRoot;
#[derive(Component, Clone, Copy)] pub(crate) struct SpawnTypeBtn(pub SoldierType);
#[derive(Component)] pub struct ToolbarButton(pub u8);

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
            ht.cities = Some(p.spawn((Text::new("城 0"), TextFont { font: font.clone(), font_size: 14.0, ..default() })).id());
            ht.pop = Some(p.spawn((Text::new("兵 0/0"), TextFont { font: font.clone(), font_size: 14.0, ..default() })).id());
            ht.enemy = Some(p.spawn((Text::new("敌 0"), TextFont { font: font.clone(), font_size: 14.0, ..default() })).id());
            ht.time = Some(p.spawn((Text::new("T 0:00"), TextFont { font: font.clone(), font_size: 14.0, ..default() })).id());
        });

        root.spawn(Node { flex_grow: 1.0, ..default() }); // spacer

        // ── Bottom zone 180px ──
        root.spawn((Node { width: Val::Percent(100.0), height: Val::Px(180.0),
            flex_direction: FlexDirection::Row, ..default() }, BottomZone))
        .with_children(|bz| {
            // ▲ Left 30%: soldier panel
            {
                let ht = &mut *ht;
                let font = &font;
                bz.spawn(Node { width: Val::Percent(30.0), height: Val::Percent(100.0), ..default() })
                  .with_children(|p| {
                    p.spawn((Node { width: Val::Percent(100.0), flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(8.0)), row_gap: Val::Px(3.0), ..default() },
                        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
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
                    });
                  });
            }
            // ▲ Center 40%: city panel + command card
            {
                let ht = &mut *ht;
                let font = &font;
                bz.spawn(Node { width: Val::Percent(40.0), height: Val::Percent(100.0), flex_direction: FlexDirection::Column, ..default() })
                  .with_children(|p| {
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
                    // Command card
                    p.spawn((Node { width: Val::Percent(100.0), flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(8.0)), row_gap: Val::Px(4.0), ..default() },
                        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
                    )).with_children(|p| {
                        p.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(6.0), ..default() }).with_children(|p| {
                            for label in ["移动","攻击","停止","驻守"] {
                                p.spawn((Button, Node { padding: UiRect::all(Val::Px(8.0)), ..default() }))
                                    .with_child((Text::new(label), TextFont { font: font.clone(), font_size: 14.0, ..default() }));
                            }
                        });
                        ht.cmd_info = Some(p.spawn((Text::new("无可用命令 — 请先选择单位"), TextFont { font: font.clone(), font_size: 12.0, ..default() })).id());
                    });
                  });
            }
            // ▲ Right 30%: minimap + compendium
            {
                let ht = &mut *ht;
                let font = &font;
                bz.spawn(Node { width: Val::Percent(30.0), height: Val::Percent(100.0), flex_direction: FlexDirection::Column, ..default() })
                  .with_children(|p| {
                    p.spawn((Node { width: Val::Percent(100.0), height: Val::Percent(55.0),
                        justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                        BackgroundColor(Color::srgba(0.05, 0.05, 0.05, 0.6)),
                    )).with_child((Text::new("小地图"), TextFont { font: font.clone(), font_size: 12.0, ..default() }));
                    p.spawn((Node { width: Val::Percent(100.0), flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(8.0)), row_gap: Val::Px(3.0), ..default() },
                        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
                    )).with_children(|p| {
                        ht.comp_header = Some(p.spawn((Text::new("悬停兵种按钮查看详情"), TextFont { font: font.clone(), font_size: 13.0, ..default() })).id());
                        for e in [&mut ht.comp_hp, &mut ht.comp_atk, &mut ht.comp_spd, &mut ht.comp_rng, &mut ht.comp_rad, &mut ht.comp_effect, &mut ht.comp_desc] {
                            *e = Some(p.spawn((Text::new(""), TextFont { font: font.clone(), font_size: 12.0, ..default() })).id());
                        }
                    });
                  });
            }
        });

        // ── Toolbar ──
        root.spawn((Node { width: Val::Percent(100.0), height: Val::Px(40.0), flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
        )).with_children(|p| {
            for (label, marker) in [("O框选",0u8),("[ ]框选",1),("盾",2),("[>]优先",3)] {
                p.spawn((Button, Node { padding: UiRect::all(Val::Px(6.0)), margin: UiRect::all(Val::Px(3.0)), ..default() }, ToolbarButton(marker)))
                    .with_child((Text::new(label), TextFont { font: font.clone(), font_size: 13.0, ..default() }));
            }
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
    mut hp_s: Query<(&mut Node, &mut BackgroundColor), With<HpFillS>>,
    mut exp_s: Query<(&mut Node, &mut BackgroundColor), With<ExpFillS>>,
    mut hp_c: Query<(&mut Node, &mut BackgroundColor), With<HpFillC>>,
    mut city_root_q: Query<&mut Node, With<CityPanelRoot>>,
    spawn_btns: Query<(&SpawnTypeBtn, &Interaction), Changed<Interaction>>,
    ht: Res<HudTexts>, sel_city: Res<SelectedCity>, selection: Res<SelectionState>,
    mut sim: bevy::ecs::system::NonSendMut<SimulationWorld>,
) {
    let w = &mut sim.0;
    let has_city = sel_city.0.is_some();
    let has_soldiers = !selection.selected_unit_ids.is_empty();

    // City panel visibility
    if let Some(root_e) = ht.c_root {
        if let Ok(mut n) = city_root_q.get_mut(root_e) {
            n.display = if has_city { Display::Flex } else { Display::None };
        }
    }

    // ── Update city panel ──
    if has_city {
        if let Some(ce) = sel_city.0 {
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
                if let Some(f) = ht.c_hp_fill { if let Ok((mut n,mut bg))=hp_c.get_mut(f) {
                    n.width=Val::Percent(r*100.0); bg.0=if r>0.5{Color::srgba(0.2,0.8,0.2,1.0)}else{Color::srgba(0.9,0.2,0.2,1.0)}; } }
            }
        }
    }

    // ── Update soldier panel ──
    if has_soldiers && !has_city {
        let ids = &selection.selected_unit_ids;
        // Collect soldier info
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
            if let Some(f) = ht.s_hp_fill { if let Ok((mut n,mut bg))=hp_s.get_mut(f) {
                let r = s.hp as f32/s.mhp.max(1) as f32;
                n.width=Val::Percent(r*100.0); bg.0=if r>0.5{Color::srgba(0.2,0.8,0.2,1.0)}else{Color::srgba(0.9,0.2,0.2,1.0)}; } }
            if let Some(f) = ht.s_exp_fill { if let Ok((mut n,_))=exp_s.get_mut(f) { n.width=Val::Percent((s.exp as f32/100.0*100.0).min(100.0)); } }
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
            if let Some(f)=ht.s_exp_fill { if let Ok((mut n,_))=exp_s.get_mut(f) { n.width=Val::Percent(0.0); } }
            let r = thp as f32/tmax.max(1) as f32;
            if let Some(f)=ht.s_hp_fill { if let Ok((mut n,mut bg))=hp_s.get_mut(f) { n.width=Val::Percent(r*100.0); bg.0=if r>0.5{Color::srgba(0.2,0.8,0.2,1.0)}else{Color::srgba(0.9,0.2,0.2,1.0)}; } }
        }
    } else {
        // No selection: show placeholder in soldier panel
        set_text(&mut tq, ht.s_header, "点击单位以查看详情");
        for e in [ht.s_hp_text,ht.s_atk,ht.s_spd,ht.s_exp_text,ht.s_effect,ht.s_origin] { if let Some(id)=e { if let Ok(mut t)=tq.get_mut(id) { t.0.clear(); } } }
        if let Some(f)=ht.s_hp_fill { if let Ok((mut n,_))=hp_s.get_mut(f) { n.width=Val::Percent(0.0); } }
        if let Some(f)=ht.s_exp_fill { if let Ok((mut n,_))=exp_s.get_mut(f) { n.width=Val::Percent(0.0); } }
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
        // Keep showing whatever was last hovered, or clear
        show_compendium(&mut tq, &ht, SoldierType::Militia); // show militia as default for city
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
    sel: Res<SelectedCity>, mut sim: bevy::ecs::system::NonSendMut<SimulationWorld>) {
    for (btn,interaction) in q.iter_mut() {
        if *interaction != Interaction::Pressed { continue; }
        if let Some(ce) = sel.0 {
            if let Some(mut c) = sim.0.entity_mut(ce).get_mut::<CityComponent>() { c.spawn_type = btn.0; }
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

pub fn city_click_system(mouse: Res<ButtonInput<MouseButton>>, qw: Query<&Window>,
    cam_q: Query<(&Camera, &GlobalTransform), With<crate::camera::MainCamera>>,
    mut sel_city: ResMut<SelectedCity>, mut sim: bevy::ecs::system::NonSendMut<SimulationWorld>) {
    if !mouse.just_pressed(MouseButton::Left) { return; }
    let Ok(w) = qw.single() else { return };
    let Some(c) = w.cursor_position() else { return };
    let Ok((cam,ct)) = cam_q.single() else { return };
    let Some(wp) = cam.viewport_to_world_2d(ct,c).ok() else { return };
    let world = &mut sim.0;
    let mut q = world.query::<(Entity,&LogicalPosition,&CityRadius,&FactionComponent)>();
    for (e,pos,radius,fac) in q.iter(world) {
        if fac.0 != Faction::Player { continue; }
        let dx = pos.0.x.to_float()-wp.x; let dy = pos.0.y.to_float()-wp.y;
        if (dx*dx+dy*dy) < (radius.0 as f32).powi(2) { sel_city.0 = Some(e); return; }
    }
}
