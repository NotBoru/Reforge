use crate::app::{ActiveModal, App, Message};
use crate::core::uninstaller::UninstallStage;
use crate::core::winapi::format_bytes;
use crate::i18n::{tr, Lang};
use crate::ui::components::{
    app_logo, destructive_button, inset_group, modal_shell, primary_button, secondary_button,
    status_dot,
};
use crate::ui::icons;
use crate::ui::motion;
use crate::ui::theme::{self, Palette, FONT_MEDIUM, FONT_REGULAR, FONT_SEMIBOLD};
use iced::alignment::Vertical;
use iced::widget::{checkbox, column, container, row, scrollable, text, Space};
use iced::{Color, Element, Length};

pub fn view<'a>(app: &'a App, modal: &'a ActiveModal, p: Palette) -> Element<'a, Message> {
    let s = tr(app.config.lang);
    let alpha = app.motion.modal_alpha();

    match modal {
        ActiveModal::ConfirmUninstall {
            apps,
            close_processes,
            force_hung,
            scan_traces_after,
            prefer_quiet,
        } => {
            let total_size: u64 = apps.iter().map(|a| a.estimated_size_bytes).sum();
            let sub = if total_size > 0 {
                format!("{} apps selected · {}", apps.len(), format_bytes(total_size))
            } else {
                format!("{} apps selected", apps.len())
            };

            let app_rows: Vec<Element<'a, Message>> = apps
                .iter()
                .map(|a| {
                    let sz = if a.estimated_size_bytes > 0 {
                        format_bytes(a.estimated_size_bytes)
                    } else {
                        String::new()
                    };
                    container(
                        row![
                            app_logo(app.installed_icon(&a.id), &a.name, 22.0, p),
                            column![
                                text(&a.name)
                                    .font(FONT_MEDIUM)
                                    .size(12.5)
                                    .color(p.text_primary),
                                text(&a.publisher)
                                    .font(FONT_REGULAR)
                                    .size(11)
                                    .color(p.text_muted),
                            ]
                            .spacing(1)
                            .width(Length::Fill),
                            text(sz)
                                .font(FONT_REGULAR)
                                .size(11.5)
                                .color(p.text_secondary),
                        ]
                        .spacing(10)
                        .align_y(Vertical::Center),
                    )
                    .padding(iced::Padding {
                        top: 8.0,
                        right: 18.0,
                        bottom: 8.0,
                        left: 12.0,
                    })
                    .into()
                })
                .collect();

            let list_box = scrollable(inset_group(p, app_rows))
                .height(Length::Fixed((apps.len() as f32 * 46.0).clamp(52.0, 210.0)))
                .style(theme::scrollable_style(p));

            let options = column![
                checkbox(*close_processes)
                    .label(s.opt_close_procs)
                    .size(15)
                    .text_size(12.5)
                    .style(theme::checkbox_style(p))
                    .on_toggle(Message::ModalToggleCloseProcs),
                checkbox(*force_hung)
                    .label(s.opt_force_hung)
                    .size(15)
                    .text_size(12.5)
                    .style(theme::checkbox_style(p))
                    .on_toggle(Message::ModalToggleForceHung),
                checkbox(*scan_traces_after)
                    .label(s.opt_scan_traces_after)
                    .size(15)
                    .text_size(12.5)
                    .style(theme::checkbox_style(p))
                    .on_toggle(Message::ModalToggleScanTracesAfter),
                checkbox(*prefer_quiet)
                    .label(s.opt_quiet_uninstall)
                    .size(15)
                    .text_size(12.5)
                    .style(theme::checkbox_style(p))
                    .on_toggle(Message::ModalToggleQuietUninstall),
            ]
            .spacing(9);

            let body = column![list_box, options].spacing(16);

            let footer = row![
                Space::new().width(Length::Fill),
                secondary_button(p, s.cancel.into(), None, Some(Message::CloseModal)),
                destructive_button(
                    p,
                    format!("{} ({})", s.uninstall_selected, apps.len()),
                    Some(icons::trash(13.0, Color::WHITE).into()),
                    Some(Message::ConfirmBulkUninstall),
                ),
            ]
            .spacing(8)
            .align_y(Vertical::Center);

            modal_shell(
                p,
                alpha,
                530.0,
                s.confirm_uninstall_title,
                sub,
                body.into(),
                footer.into(),
                Some(Message::CloseModal),
            )
        }

        ActiveModal::UninstallProgress {
            items,
            finished,
            scan_traces_after,
        } => {
            let done_count = items
                .iter()
                .filter(|i| {
                    matches!(
                        i.stage,
                        UninstallStage::Removed { .. }
                            | UninstallStage::StillPresent(_)
                            | UninstallStage::Failed(_)
                    )
                })
                .count();
            let sub = format!("{done_count} / {}", items.len());

            let rows: Vec<Element<'a, Message>> = items
                .iter()
                .map(|it| {
                    let status_el: Element<'a, Message> = match &it.stage {
                        UninstallStage::Queued => text("Queued")
                            .font(FONT_REGULAR)
                            .size(11.5)
                            .color(p.text_muted)
                            .into(),
                        UninstallStage::ClosingProcesses(n) => row![
                            motion::spinner(app.motion.spinner_phase, 13.0, p.accent, p.hairline),
                            text(format!("Closing {n} process(es)..."))
                                .font(FONT_MEDIUM)
                                .size(11.5)
                                .color(p.text_secondary),
                        ]
                        .spacing(7)
                        .align_y(Vertical::Center)
                        .into(),
                        UninstallStage::RunningUninstaller => row![
                            motion::spinner(app.motion.spinner_phase, 13.0, p.accent, p.hairline),
                            text("Uninstalling...")
                                .font(FONT_MEDIUM)
                                .size(11.5)
                                .color(p.accent),
                        ]
                        .spacing(7)
                        .align_y(Vertical::Center)
                        .into(),
                        UninstallStage::Verifying => row![
                            motion::spinner(app.motion.spinner_phase, 13.0, p.accent, p.hairline),
                            text("Verifying...")
                                .font(FONT_MEDIUM)
                                .size(11.5)
                                .color(p.text_secondary),
                        ]
                        .spacing(7)
                        .align_y(Vertical::Center)
                        .into(),
                        UninstallStage::Removed { reboot_required } => status_dot(
                            p.success,
                            Some(if *reboot_required {
                                "Removed (reboot)"
                            } else {
                                "Removed"
                            }),
                            p,
                        ),
                        UninstallStage::StillPresent(_) => {
                            status_dot(p.warning, Some("Still registered"), p)
                        }
                        UninstallStage::Failed(_) => status_dot(p.danger, Some("Failed"), p),
                    };

                    let mut col = column![row![
                        app_logo(app.installed_icon(&it.app_id), &it.app_name, 22.0, p),
                        text(&it.app_name)
                            .font(FONT_MEDIUM)
                            .size(12.5)
                            .color(p.text_primary)
                            .width(Length::Fill),
                        status_el,
                    ]
                    .spacing(10)
                    .align_y(Vertical::Center)]
                    .spacing(3);

                    if let UninstallStage::StillPresent(msg) | UninstallStage::Failed(msg) =
                        &it.stage
                    {
                        col = col.push(
                            text(msg)
                                .font(FONT_REGULAR)
                                .size(11)
                                .color(p.text_muted),
                        );
                    }

                    container(col).padding([9, 12]).into()
                })
                .collect();

            let body = scrollable(inset_group(p, rows))
                .height(Length::Fixed((items.len() as f32 * 52.0).clamp(72.0, 280.0)))
                .style(theme::scrollable_style(p));

            let mut footer = row![Space::new().width(Length::Fill)]
                .spacing(8)
                .align_y(Vertical::Center);
            if *finished {
                if *scan_traces_after {
                    footer = footer.push(primary_button(
                        p,
                        s.scan_traces.into(),
                        Some(icons::search(13.0, Color::WHITE).into()),
                        Some(Message::ProceedToBatchTraceScan),
                    ));
                }
                footer = footer.push(secondary_button(
                    p,
                    s.done.into(),
                    None,
                    Some(Message::CloseModal),
                ));
            } else {
                footer = footer.push(
                    row![
                        motion::spinner(app.motion.spinner_phase, 14.0, p.accent, p.hairline),
                        text(s.working)
                            .font(FONT_MEDIUM)
                            .size(12)
                            .color(p.text_secondary),
                    ]
                    .spacing(8)
                    .align_y(Vertical::Center),
                );
            }

            modal_shell(
                p,
                alpha,
                550.0,
                s.uninstall_progress_title,
                sub,
                body.into(),
                footer.into(),
                if *finished {
                    Some(Message::CloseModal)
                } else {
                    None
                },
            )
        }

        ActiveModal::AppDetailsInstalled(inst) => {
            let meta_row = |label: &'static str, val: String| -> Element<'a, Message> {
                container(
                    row![
                        text(label)
                            .font(FONT_MEDIUM)
                            .size(12)
                            .color(p.text_secondary)
                            .width(Length::Fixed(120.0)),
                        text(if val.is_empty() { "—".into() } else { val })
                            .font(FONT_REGULAR)
                            .size(12)
                            .color(p.text_primary)
                            .width(Length::Fill),
                    ]
                    .spacing(12)
                    .align_y(Vertical::Center),
                )
                .padding([9, 12])
                .into()
            };

            let top = row![
                app_logo(app.installed_icon(&inst.id), &inst.name, 40.0, p),
                column![
                    text(&inst.name)
                        .font(FONT_SEMIBOLD)
                        .size(15)
                        .color(p.text_primary),
                    text(format!(
                        "{}{}",
                        inst.publisher,
                        if inst.version.is_empty() {
                            String::new()
                        } else {
                            format!("  ·  v{}", inst.version)
                        }
                    ))
                    .font(FONT_REGULAR)
                    .size(12)
                    .color(p.text_secondary),
                ]
                .spacing(2),
            ]
            .spacing(12)
            .align_y(Vertical::Center);

            let info = inset_group(
                p,
                vec![
                    meta_row(
                        "Size",
                        if inst.estimated_size_bytes > 0 {
                            format_bytes(inst.estimated_size_bytes)
                        } else {
                            "—".into()
                        },
                    ),
                    meta_row("Installed", inst.install_date.clone()),
                    meta_row("Location", inst.install_location.clone()),
                    meta_row(
                        "Registry",
                        format!("{}\\{}", inst.reg_root.as_str(), inst.reg_subkey),
                    ),
                ],
            );

            let body = column![top, info].spacing(14);

            let mut footer = row![].spacing(8).align_y(Vertical::Center);
            if !inst.install_location.is_empty() {
                footer = footer.push(secondary_button(
                    p,
                    s.open_folder.into(),
                    Some(icons::folder(13.0, p.text_secondary).into()),
                    Some(Message::OpenPath(inst.install_location.clone())),
                ));
            }
            footer = footer
                .push(secondary_button(
                    p,
                    s.scan_traces.into(),
                    Some(icons::search(13.0, p.accent).into()),
                    Some(Message::DeepScanApps(vec![inst.clone()])),
                ))
                .push(Space::new().width(Length::Fill))
                .push(secondary_button(
                    p,
                    s.force_remove.into(),
                    None,
                    Some(Message::ForceRemoveApp(inst.clone())),
                ))
                .push(primary_button(
                    p,
                    s.close.into(),
                    None,
                    Some(Message::CloseModal),
                ));

            modal_shell(
                p,
                alpha,
                580.0,
                s.details,
                inst.publisher.clone(),
                body.into(),
                footer.into(),
                Some(Message::CloseModal),
            )
        }

        ActiveModal::AppDetailsCatalog(cat_app) => {
            let desc = match app.config.lang {
                Lang::En => cat_app.desc_en,
                Lang::PtBr => cat_app.desc_pt,
            };
            let is_inst = app.is_catalog_app_installed(cat_app);
            let selected = app.selected_catalog_ids.contains(cat_app.id);

            let top = row![
                app_logo(app.catalog_icon(cat_app), cat_app.name, 40.0, p),
                column![
                    text(cat_app.name)
                        .font(FONT_SEMIBOLD)
                        .size(15)
                        .color(p.text_primary),
                    text(cat_app.publisher)
                        .font(FONT_REGULAR)
                        .size(12)
                        .color(p.text_secondary),
                ]
                .spacing(2),
            ]
            .spacing(12)
            .align_y(Vertical::Center);

            let meta_row = |label: &'static str, val: String| -> Element<'a, Message> {
                container(
                    row![
                        text(label)
                            .font(FONT_MEDIUM)
                            .size(12)
                            .color(p.text_secondary)
                            .width(Length::Fixed(115.0)),
                        text(val)
                            .font(FONT_REGULAR)
                            .size(12)
                            .color(p.text_primary)
                            .width(Length::Fill),
                    ]
                    .spacing(12),
                )
                .padding([9, 12])
                .into()
            };

            let info = inset_group(
                p,
                vec![
                    meta_row("WinGet ID", cat_app.winget_id.to_string()),
                    meta_row("Website", cat_app.domain.to_string()),
                    meta_row(
                        "Delivery",
                        if cat_app.direct.is_some() {
                            "Multi-stream Direct + WinGet fallback".into()
                        } else {
                            "WinGet Silent Package".into()
                        },
                    ),
                    meta_row(
                        "Status",
                        if is_inst {
                            s.installed.into()
                        } else {
                            "Not installed".into()
                        },
                    ),
                ],
            );

            let body = column![
                top,
                text(desc)
                    .font(FONT_REGULAR)
                    .size(12.5)
                    .color(p.text_secondary),
                info,
            ]
            .spacing(12);

            let footer = row![
                secondary_button(
                    p,
                    "Website".into(),
                    Some(icons::globe(13.0, p.text_secondary).into()),
                    Some(Message::OpenUrl(format!("https://{}", cat_app.domain))),
                ),
                Space::new().width(Length::Fill),
                secondary_button(p, s.close.into(), None, Some(Message::CloseModal)),
                primary_button(
                    p,
                    if selected {
                        s.deselect_all.into()
                    } else {
                        "Select for Install".into()
                    },
                    None,
                    Some(Message::ToggleCatalogApp(cat_app.id)),
                ),
            ]
            .spacing(8)
            .align_y(Vertical::Center);

            modal_shell(
                p,
                alpha,
                500.0,
                cat_app.name,
                cat_app.publisher.into(),
                body.into(),
                footer.into(),
                Some(Message::CloseModal),
            )
        }

        ActiveModal::ConfirmTweaks {
            apply_recommended,
            affected_count,
            create_restore_point,
        } => {
            let title = if *apply_recommended {
                s.apply_recommended
            } else {
                s.restore_defaults
            };
            let desc = if *apply_recommended {
                format!(
                    "This will enable {affected_count} recommended reversible Windows privacy, performance, and interface settings."
                )
            } else {
                format!(
                    "This will revert {affected_count} modified settings back to standard Windows defaults."
                )
            };

            let body = column![
                text(desc)
                    .font(FONT_REGULAR)
                    .size(13)
                    .color(p.text_secondary),
                checkbox(*create_restore_point)
                    .label(s.opt_restore_point_first)
                    .size(15)
                    .text_size(12.5)
                    .style(theme::checkbox_style(p))
                    .on_toggle(Message::ModalToggleRestorePoint),
            ]
            .spacing(14);

            let footer = row![
                Space::new().width(Length::Fill),
                secondary_button(p, s.cancel.into(), None, Some(Message::CloseModal)),
                primary_button(
                    p,
                    title.into(),
                    None,
                    Some(Message::ExecuteConfirmedTweaks),
                ),
            ]
            .spacing(8);

            modal_shell(
                p,
                alpha,
                470.0,
                s.confirm_tweaks_title,
                format!("{affected_count} settings"),
                body.into(),
                footer.into(),
                Some(Message::CloseModal),
            )
        }

        ActiveModal::ConfirmDebloat { packages } => {
            let rows: Vec<Element<'a, Message>> = packages
                .iter()
                .map(|pkg| {
                    container(
                        row![
                            text(&pkg.display_name)
                                .font(FONT_MEDIUM)
                                .size(12.5)
                                .color(p.text_primary)
                                .width(Length::Fill),
                            text(&pkg.package_name)
                                .font(FONT_REGULAR)
                                .size(11)
                                .color(p.text_muted),
                        ]
                        .spacing(10),
                    )
                    .padding(iced::Padding {
                        top: 8.0,
                        right: 18.0,
                        bottom: 8.0,
                        left: 12.0,
                    })
                    .into()
                })
                .collect();

            let body = column![
                text(s.debloat_safety_note)
                    .font(FONT_REGULAR)
                    .size(12)
                    .color(p.text_secondary),
                scrollable(inset_group(p, rows))
                    .height(Length::Fixed((packages.len() as f32 * 38.0).clamp(60.0, 220.0)))
                    .style(theme::scrollable_style(p)),
            ]
            .spacing(12);

            let footer = row![
                Space::new().width(Length::Fill),
                secondary_button(p, s.cancel.into(), None, Some(Message::CloseModal)),
                destructive_button(
                    p,
                    format!("{} ({})", s.remove_selected, packages.len()),
                    Some(icons::trash(13.0, Color::WHITE).into()),
                    Some(Message::ExecuteConfirmedDebloat),
                ),
            ]
            .spacing(8);

            modal_shell(
                p,
                alpha,
                510.0,
                s.confirm_debloat_title,
                format!("{} packages", packages.len()),
                body.into(),
                footer.into(),
                Some(Message::CloseModal),
            )
        }
    }
}
