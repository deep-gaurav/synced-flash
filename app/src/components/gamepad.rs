use leptos::{component, prelude::*, view, IntoView};
use svg::node::element::path::Data;
use svg::node::element::Path;
use svg::Document;
use tracing::info;

#[component]
pub fn Gamepad() -> impl IntoView {
    let (svg_w, svg_h) = (100.0, 100.0);

    let h = 40.0 / 100.0 * svg_h;

    let angle1 = 10_f64.to_radians();
    let hypo = svg_w;

    let e_y = angle1.sin() * hypo;
    let e_x = angle1.cos() * hypo;

    let extra = ((svg_w.powi(2) - (h / 2_f64 + e_y).powi(2)).sqrt() - svg_w).abs();

    let calculated_angle = (-e_y / ((-e_x + (extra * 2.0)) / 2.0)).tanh();

    // Left Fan
    let data = Data::new()
        .move_to((svg_w / 2.0, svg_h / 2.0))
        .line_by((0, -h / 2.0))
        .line_by(((-e_x + (extra * 2.0)) / 2.0, -e_y))
        .elliptical_arc_by((svg_w, svg_w, 0, 0, 0, 0, h + 2.0 * e_y))
        // .line_by(())
        .line_by(((e_x - (extra * 2.0)) / 2.0, -e_y))
        .line_by((0, -h / 2.0))
        .close();

    let path = Path::new()
        .set("fill", "none")
        .set("stroke", "black")
        .set("stroke-width", 1)
        .set("d", data);

    // Right Fan
    let data_r = Data::new()
        .move_to((svg_w / 2.0, svg_h / 2.0))
        .line_by((0, -h / 2.0))
        .line_by(((e_x - (extra * 2.0)) / 2.0, -e_y))
        .elliptical_arc_by((svg_w, svg_w, 0, 0, 1, 0, h + 2.0 * e_y))
        // .line_by(())
        .line_by(((-e_x + (extra * 2.0)) / 2.0, -e_y))
        .line_by((0, -h / 2.0))
        .close();

    let path_r = Path::new()
        .set("fill", "none")
        .set("stroke", "black")
        .set("stroke-width", 1)
        .set("d", data_r);

    // Top Fan
    let tf_angle = 50_f64.to_radians();

    let th = (svg_h - h) / 2.0;

    let (tx, ty) = (tf_angle.sin() * th, tf_angle.cos() * th);

    let data_t = Data::new()
        .move_to((svg_w / 2.0, svg_h / 2.0 - h / 2.0))
        .line_by((tx, -ty))
        .elliptical_arc_by((th, th, 0, 0, 0, -2.0 * tx, 0))
        // .line_by(())
        // .line_by(((-e_x + (extra * 2.0)) / 2.0, -e_y))
        // .line_by((0, -h / 2.0))
        .close();

    let path_t = Path::new()
        .set("fill", "none")
        .set("stroke", "black")
        .set("stroke-width", 1)
        .set("d", data_t);

    // Top Left Fan

    let tdh = th * 0.8;

    let (tx1, ty1) = (calculated_angle.cos() * tdh, calculated_angle.sin() * tdh);
    let extra_angle = (90_f64.to_radians() - calculated_angle - tf_angle);
    let (tx2, ty2) = (
        (extra_angle + calculated_angle).cos() * tdh,
        (extra_angle + calculated_angle).sin() * tdh,
    );

    info!(
        "Ca {} tf {} ca {}",
        calculated_angle.to_degrees(),
        tf_angle.to_degrees(),
        90_f64 - calculated_angle.to_degrees() - tf_angle.to_degrees()
    );

    info!("tx2 {tx2} ty2 {ty2}");

    let data_tl = Data::new()
        .move_to((svg_w / 2.0, svg_h / 2.0 - h / 2.0))
        .line_by((-tx1, -ty1))
        .elliptical_arc_to((
            tdh,
            tdh,
            0,
            0,
            1,
            svg_w / 2.0 - tx2,
            svg_h / 2.0 - h / 2.0 - ty2,
        ))
        // .line_by(())
        // .line_by(((-e_x + (extra * 2.0)) / 2.0, -e_y))
        // .line_by((0, -h / 2.0))
        .close();

    let path_tl = Path::new()
        .set("fill", "none")
        .set("stroke", "red")
        .set("stroke-width", 1)
        .set("d", data_tl);

    let document = Document::new()
        .set("viewBox", (0, 0, svg_w, svg_h))
        .add(path)
        .add(path_r)
        .add(path_t)
        .add(path_tl);

    view! {
        <div class="fixed z-50 top-0 left-0 w-full h-full flex items-center justify-center bg-white">
            <div class="w-40 h-40" inner_html={document.to_string()} />
        </div>
    }
}
