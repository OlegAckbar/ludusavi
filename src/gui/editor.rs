use std::collections::HashSet;

use iced::{
    keyboard, padding,
    widget::{horizontal_space, tooltip, Space},
    Alignment, Length,
};

use crate::{
    gui::{
        badge::Badge,
        button,
        common::{BackupPhase, BrowseFileSubject, BrowseSubject, Message, ScrollSubject, UndoSubject},
        icon::Icon,
        search::CustomGamesFilter,
        shortcuts::TextHistories,
        style,
        widget::{checkbox, pick_list, text, Column, Container, IcedParentExt, Row, Tooltip},
    },
    lang::TRANSLATOR,
    resource::{
        cache::Cache,
        config::{Config, CustomGameKind, Integration, RedirectKind, SecondaryManifestConfigKind},
        manifest::{Manifest, Store},
    },
};

pub fn root<'a>(config: &Config, histories: &TextHistories, modifiers: &keyboard::Modifiers) -> Container<'a> {
    let mut content = Column::new().width(Length::Fill).spacing(5);
    if config.roots.is_empty() {
        content = content.push(text(TRANSLATOR.no_roots_are_configured()));
    } else {
        content = config
            .roots
            .iter()
            .enumerate()
            .fold(content, |parent, (i, root)| match root.store() {
                Store::Lutris => parent
                    .push(
                        Row::new()
                            .spacing(20)
                            .push(button::move_up(Message::EditedRoot, i))
                            .push(button::move_down(Message::EditedRoot, i, config.roots.len()))
                            .push(histories.input(UndoSubject::RootPath(i)))
                            .push(
                                pick_list(Store::ALL, Some(root.store()), move |v| {
                                    Message::SelectedRootStore(i, v)
                                })
                                .class(style::PickList::Primary),
                            )
                            .push(button::choose_folder(BrowseSubject::Root(i), modifiers))
                            .push(button::remove(Message::EditedRoot, i)),
                    )
                    .push(
                        Row::new()
                            .spacing(20)
                            .align_y(Alignment::Center)
                            .push(horizontal_space().width(70))
                            .push(text(TRANSLATOR.field("pga.db")))
                            .push(histories.input(UndoSubject::RootLutrisDatabase(i)))
                            .push(button::choose_file(BrowseFileSubject::RootLutrisDatabase(i), modifiers)),
                    ),
                _ => parent.push(
                    Row::new()
                        .spacing(20)
                        .push(button::move_up(Message::EditedRoot, i))
                        .push(button::move_down(Message::EditedRoot, i, config.roots.len()))
                        .push(histories.input(UndoSubject::RootPath(i)))
                        .push(
                            pick_list(Store::ALL, Some(root.store()), move |v| {
                                Message::SelectedRootStore(i, v)
                            })
                            .class(style::PickList::Primary),
                        )
                        .push(button::choose_folder(BrowseSubject::Root(i), modifiers))
                        .push(button::remove(Message::EditedRoot, i)),
                ),
            });
    };

    content = content.push(
        Row::new()
            .spacing(20)
            .push(button::add(Message::EditedRoot))
            .push(button::search(Message::FindRoots)),
    );

    Container::new(content)
}

pub fn manifest<'a>(
    config: &Config,
    cache: &'a Cache,
    histories: &TextHistories,
    modifiers: &keyboard::Modifiers,
) -> Container<'a> {
    let label_width = Length::Fixed(160.0);
    let right_offset = Length::Fixed(70.0);

    let get_checked = |url: Option<&str>, cache: &'a Cache| {
        let url = url?;
        let cached = cache.manifests.get(url)?;
        let checked = match cached.checked {
            Some(x) => chrono::DateTime::<chrono::Local>::from(x)
                .format("%Y-%m-%dT%H:%M:%S")
                .to_string(),
            None => "?".to_string(),
        };
        Some(Container::new(text(checked)).width(label_width))
    };

    let get_updated = |url: Option<&str>, cache: &'a Cache| {
        let url = url?;
        let cached = cache.manifests.get(url)?;
        let updated = match cached.updated {
            Some(x) => chrono::DateTime::<chrono::Local>::from(x)
                .format("%Y-%m-%dT%H:%M:%S")
                .to_string(),
            None => "?".to_string(),
        };
        Some(Container::new(text(updated)).width(label_width))
    };

    let mut content = Column::new()
        .padding(5)
        .spacing(5)
        .push(
            Row::new()
                .spacing(20)
                .align_y(Alignment::Center)
                .push(Space::with_width(Length::Fill))
                .push(Container::new(text(TRANSLATOR.checked_label())).width(label_width))
                .push(Container::new(text(TRANSLATOR.updated_label())).width(label_width))
                .push_if(!config.manifest.secondary.is_empty(), || {
                    Space::with_width(right_offset)
                }),
        )
        .push(
            Row::new()
                .spacing(20)
                .align_y(Alignment::Center)
                .push(
                    checkbox("", config.manifest.enable, move |enabled| {
                        Message::TogglePrimaryManifestEnabled { enabled }
                    })
                    .spacing(0)
                    .class(style::Checkbox),
                )
                .push(iced::widget::TextInput::new("", config.manifest.url()).width(Length::Fill))
                .push_maybe(get_checked(Some(config.manifest.url()), cache))
                .push_maybe(get_updated(Some(config.manifest.url()), cache))
                .push_if(!config.manifest.secondary.is_empty(), || {
                    Space::with_width(right_offset)
                }),
        );

    content = config
        .manifest
        .secondary
        .iter()
        .enumerate()
        .fold(content, |column, (i, _)| {
            column.push(
                Row::new()
                    .spacing(20)
                    .align_y(Alignment::Center)
                    .push(
                        checkbox("", config.manifest.secondary[i].enabled(), move |enabled| {
                            Message::ToggleSecondaryManifestEnabled { index: i, enabled }
                        })
                        .spacing(0)
                        .class(style::Checkbox),
                    )
                    .push(button::move_up(Message::EditedSecondaryManifest, i))
                    .push(button::move_down(
                        Message::EditedSecondaryManifest,
                        i,
                        config.manifest.secondary.len(),
                    ))
                    .push(
                        pick_list(
                            SecondaryManifestConfigKind::ALL,
                            Some(config.manifest.secondary[i].kind()),
                            move |v| Message::SelectedSecondaryManifestKind(i, v),
                        )
                        .class(style::PickList::Primary)
                        .width(75),
                    )
                    .push(histories.input(UndoSubject::SecondaryManifest(i)))
                    .push_maybe(get_checked(config.manifest.secondary[i].url(), cache))
                    .push_maybe(get_updated(config.manifest.secondary[i].url(), cache))
                    .push_maybe(match config.manifest.secondary[i].kind() {
                        SecondaryManifestConfigKind::Local => {
                            Some(button::choose_file(BrowseFileSubject::SecondaryManifest(i), modifiers))
                        }
                        SecondaryManifestConfigKind::Remote => None,
                    })
                    .push(button::remove(Message::EditedSecondaryManifest, i)),
            )
        });

    content = content.push(button::add(Message::EditedSecondaryManifest));

    Container::new(content).class(style::Container::GameListEntry)
}

pub fn redirect<'a>(config: &Config, histories: &TextHistories, modifiers: &keyboard::Modifiers) -> Container<'a> {
    let redirects = config.get_redirects();

    let wrapper = Container::new({
        let mut content = Column::new().padding(5).spacing(4).push(checkbox(
            TRANSLATOR.reverse_redirects_when_restoring(),
            config.restore.reverse_redirects,
            Message::EditedReverseRedirectsOnRestore,
        ));

        content = config.redirects.iter().enumerate().fold(content, |parent, (i, _)| {
            parent.push(
                Row::new()
                    .spacing(20)
                    .push(button::move_up(|x| Message::EditedRedirect(x, None), i))
                    .push(button::move_down(
                        |x| Message::EditedRedirect(x, None),
                        i,
                        config.redirects.len(),
                    ))
                    .push(
                        pick_list(RedirectKind::ALL, Some(redirects[i].kind), move |v| {
                            Message::SelectedRedirectKind(i, v)
                        })
                        .class(style::PickList::Primary),
                    )
                    .push(histories.input(UndoSubject::RedirectSource(i)))
                    .push(button::choose_folder(BrowseSubject::RedirectSource(i), modifiers))
                    .push(histories.input(UndoSubject::RedirectTarget(i)))
                    .push(button::choose_folder(BrowseSubject::RedirectTarget(i), modifiers))
                    .push(button::remove(|x| Message::EditedRedirect(x, None), i)),
            )
        });

        content.push(button::add(|x| Message::EditedRedirect(x, None)))
    })
    .class(style::Container::GameListEntry);

    Container::new(wrapper)
}

pub fn custom_games<'a>(
    config: &Config,
    manifest: &Manifest,
    operating: bool,
    histories: &TextHistories,
    modifiers: &keyboard::Modifiers,
    filter: &CustomGamesFilter,
) -> Container<'a> {
    if config.custom_games.is_empty() {
        return Container::new(Space::new(Length::Shrink, Length::Shrink));
    }

    let content = config.custom_games.iter().enumerate().fold(
        Column::new()
            .width(Length::Fill)
            .padding(padding::top(0).bottom(5).left(15).right(15))
            .spacing(10),
        |parent, (i, x)| {
            if !filter.qualifies(x) {
                return parent;
            }
            parent.push({
                let mut content = Column::new().padding(5).spacing(5).push(
                    Row::new()
                        .spacing(20)
                        .align_y(iced::Alignment::Center)
                        .push(button::expand(
                            x.expanded,
                            Message::ToggleCustomGameExpanded {
                                index: i,
                                expanded: !x.expanded,
                            },
                        ))
                        .push(
                            Row::new()
                                .width(110)
                                .spacing(20)
                                .align_y(Alignment::Center)
                                .push(
                                    checkbox("", config.is_custom_game_enabled(i), move |enabled| {
                                        Message::ToggleCustomGameEnabled { index: i, enabled }
                                    })
                                    .spacing(0)
                                    .class(style::Checkbox),
                                )
                                .push(button::move_up_maybe(Message::EditedCustomGame, i, !filter.enabled))
                                .push(button::move_down_maybe(
                                    Message::EditedCustomGame,
                                    i,
                                    config.custom_games.len(),
                                    !filter.enabled,
                                )),
                        )
                        .push(histories.input(UndoSubject::CustomGameName(i)))
                        .push_maybe(if manifest.0.get(&x.name).is_some_and(|game| game.is_from_manifest()) {
                            Some(match x.effective_integration() {
                                Integration::Override => Badge::icon(Icon::CallSplit)
                                    .tooltip(TRANSLATOR.custom_game_will_override())
                                    .view(),
                                Integration::Extend => Badge::icon(Icon::CallMerge)
                                    .tooltip(TRANSLATOR.custom_game_will_extend())
                                    .view(),
                            })
                        } else {
                            None
                        })
                        .push(
                            pick_list(CustomGameKind::ALL, Some(config.custom_games[i].kind()), move |v| {
                                Message::SelectedCustomGameKind(i, v)
                            })
                            .class(style::PickList::Primary)
                            .width(100),
                        )
                        .push(
                            Tooltip::new(
                                button::refresh_custom_game(
                                    Message::Backup(BackupPhase::Start {
                                        games: Some(HashSet::from([config.custom_games[i].name.clone()])),
                                        preview: true,
                                        jump: true,
                                        repair: false,
                                    }),
                                    operating,
                                    config.is_custom_game_individually_scannable(i),
                                ),
                                text(TRANSLATOR.preview_button_in_custom_mode()).size(16),
                                tooltip::Position::Top,
                            )
                            .gap(5)
                            .class(style::Container::Tooltip),
                        )
                        .push(button::delete(Message::EditedCustomGame, i)),
                );

                if x.expanded {
                    let top_side = 5;
                    let left_side = 165;

                    content = content
                        .push_if(config.custom_games[i].kind() == CustomGameKind::Alias, move || {
                            Row::new()
                                .spacing(10)
                                .align_y(Alignment::Center)
                                .push(
                                    Column::new()
                                        .width(left_side)
                                        .padding(padding::top(top_side))
                                        .push(text(TRANSLATOR.original_name_field())),
                                )
                                .push(histories.input(UndoSubject::CustomGameAlias(i)))
                        })
                        .push_if(config.custom_games[i].kind() == CustomGameKind::Alias, || {
                            Row::new()
                                .spacing(10)
                                .push(
                                    Container::new(horizontal_space().width(left_side)).padding(padding::top(top_side)),
                                )
                                .push(checkbox(
                                    TRANSLATOR.prefer_alias_display(),
                                    config.custom_games[i].prefer_alias,
                                    move |x| Message::EditedCustomGaleAliasDisplay(i, x),
                                ))
                        })
                        .push_if(config.custom_games[i].kind() == CustomGameKind::Game, || {
                            Row::new()
                                .spacing(10)
                                .push(
                                    Column::new()
                                        .width(left_side)
                                        .padding(padding::top(top_side))
                                        .push(text(TRANSLATOR.field(&TRANSLATOR.integration_label()))),
                                )
                                .push(
                                    pick_list(Integration::ALL, Some(config.custom_games[i].integration), move |v| {
                                        Message::SelectedCustomGameIntegration(i, v)
                                    })
                                    .class(style::PickList::Primary),
                                )
                        })
                        .push_if(config.custom_games[i].kind() == CustomGameKind::Game, || {
                            Row::new()
                                .spacing(10)
                                .push(
                                    Column::new()
                                        .width(left_side)
                                        .padding(padding::top(top_side))
                                        .push(text(TRANSLATOR.custom_files_label())),
                                )
                                .push(
                                    x.files
                                        .iter()
                                        .enumerate()
                                        .fold(Column::new().spacing(4), |column, (ii, _)| {
                                            column.push(
                                                Row::new()
                                                    .align_y(Alignment::Center)
                                                    .spacing(20)
                                                    .push(button::move_up_nested(Message::EditedCustomGameFile, i, ii))
                                                    .push(button::move_down_nested(
                                                        Message::EditedCustomGameFile,
                                                        i,
                                                        ii,
                                                        x.files.len(),
                                                    ))
                                                    .push(histories.input(UndoSubject::CustomGameFile(i, ii)))
                                                    .push(button::choose_folder(
                                                        BrowseSubject::CustomGameFile(i, ii),
                                                        modifiers,
                                                    ))
                                                    .push(button::remove_nested(Message::EditedCustomGameFile, i, ii)),
                                            )
                                        })
                                        .push(button::add_nested(Message::EditedCustomGameFile, i)),
                                )
                        })
                        .push_if(config.custom_games[i].kind() == CustomGameKind::Game, || {
                            Row::new()
                                .spacing(10)
                                .push(
                                    Column::new()
                                        .width(left_side)
                                        .padding(padding::top(top_side))
                                        .push(text(TRANSLATOR.custom_registry_label())),
                                )
                                .push(
                                    x.registry
                                        .iter()
                                        .enumerate()
                                        .fold(Column::new().spacing(4), |column, (ii, _)| {
                                            column.push(
                                                Row::new()
                                                    .spacing(20)
                                                    .align_y(Alignment::Center)
                                                    .push(button::move_up_nested(
                                                        Message::EditedCustomGameRegistry,
                                                        i,
                                                        ii,
                                                    ))
                                                    .push(button::move_down_nested(
                                                        Message::EditedCustomGameRegistry,
                                                        i,
                                                        ii,
                                                        x.registry.len(),
                                                    ))
                                                    .push(histories.input(UndoSubject::CustomGameRegistry(i, ii)))
                                                    .push(button::remove_nested(
                                                        Message::EditedCustomGameRegistry,
                                                        i,
                                                        ii,
                                                    )),
                                            )
                                        })
                                        .push(button::add_nested(Message::EditedCustomGameRegistry, i)),
                                )
                        });
                }

                Container::new(content)
                    .id(iced::widget::container::Id::new(config.custom_games[i].name.clone()))
                    .class(style::Container::GameListEntry)
            })
        },
    );

    Container::new(ScrollSubject::CustomGames.into_widget(content))
}

pub fn ignored_items<'a>(config: &Config, histories: &TextHistories, modifiers: &keyboard::Modifiers) -> Container<'a> {
    Container::new({
        Column::new().spacing(10).push(
            Container::new(
                Column::new()
                    .padding(5)
                    .spacing(5)
                    .push(
                        Row::new()
                            .push(Column::new().width(100).push(text(TRANSLATOR.custom_files_label())))
                            .push(
                                config
                                    .backup
                                    .filter
                                    .ignored_paths
                                    .iter()
                                    .enumerate()
                                    .fold(Column::new().spacing(4), |column, (ii, _)| {
                                        column.push(
                                            Row::new()
                                                .spacing(20)
                                                .push(button::move_up(Message::EditedBackupFilterIgnoredPath, ii))
                                                .push(button::move_down(
                                                    Message::EditedBackupFilterIgnoredPath,
                                                    ii,
                                                    config.backup.filter.ignored_paths.len(),
                                                ))
                                                .push(histories.input(UndoSubject::BackupFilterIgnoredPath(ii)))
                                                .push(button::choose_folder(
                                                    BrowseSubject::BackupFilterIgnoredPath(ii),
                                                    modifiers,
                                                ))
                                                .push(button::remove(Message::EditedBackupFilterIgnoredPath, ii)),
                                        )
                                    })
                                    .push(button::add(Message::EditedBackupFilterIgnoredPath)),
                            ),
                    )
                    .push(
                        Row::new()
                            .push(Column::new().width(100).push(text(TRANSLATOR.custom_registry_label())))
                            .push(
                                config
                                    .backup
                                    .filter
                                    .ignored_registry
                                    .iter()
                                    .enumerate()
                                    .fold(Column::new().spacing(4), |column, (ii, _)| {
                                        column.push(
                                            Row::new()
                                                .spacing(20)
                                                .push(button::move_up(Message::EditedBackupFilterIgnoredRegistry, ii))
                                                .push(button::move_down(
                                                    Message::EditedBackupFilterIgnoredRegistry,
                                                    ii,
                                                    config.backup.filter.ignored_registry.len(),
                                                ))
                                                .push(histories.input(UndoSubject::BackupFilterIgnoredRegistry(ii)))
                                                .push(button::remove(Message::EditedBackupFilterIgnoredRegistry, ii)),
                                        )
                                    })
                                    .push(button::add(Message::EditedBackupFilterIgnoredRegistry)),
                            ),
                    ),
            )
            .class(style::Container::GameListEntry),
        )
    })
}
