use charts::{AxisPosition, Chart, ScaleBand, ScaleLinear, VerticalBarView};
use lazy_static::lazy_static;
use subvt_config::Config;
use subvt_types::substrate::Balance;
use subvt_utility::numeric::format_decimal;

lazy_static! {
    static ref CONFIG: Config = Config::default();
}

pub fn plot_validator_monthly_rewards(rewards: &[(u8, u32, Balance)]) {
    let months = vec![
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let mut domain = vec![];
    let mut data = vec![];
    let mut total = 0;
    for reward in rewards {
        let tick = format!("{} {}", months[reward.0 as usize - 1], reward.1 % 100);
        domain.push(tick.clone());
        let amount: f32 = format_decimal(
            reward.2,
            CONFIG.substrate.token_decimals,
            CONFIG.substrate.token_format_decimal_points,
        )
        .parse()
        .unwrap();
        data.push((tick, amount));
        total += reward.2;
    }
    let total = format_decimal(
        total,
        CONFIG.substrate.token_decimals,
        CONFIG.substrate.token_format_decimal_points,
    );
    let max = (format_decimal(
        rewards.iter().map(|reward| reward.2).max().unwrap(),
        CONFIG.substrate.token_decimals,
        CONFIG.substrate.token_format_decimal_points,
    )
    .parse::<f32>()
    .unwrap()
        * 1.2)
        .ceil();

    // Define chart related sizes.
    let width = 1200;
    let height = 600;
    let (top, right, bottom, left) = (50, 40, 50, 60);

    // Create a band scale that maps ["A", "B", "C"] categories to values in the [0, availableWidth]
    // range (the width of the chart without the margins).
    let x = ScaleBand::new()
        .set_domain(domain)
        .set_range(vec![0, width - left - right])
        .set_inner_padding(0.1)
        .set_outer_padding(0.1);

    // Create a linear scale that will interpolate values in [0, 100] range to corresponding
    // values in [availableHeight, 0] range (the height of the chart without the margins).
    // The [availableHeight, 0] range is inverted because SVGs coordinate system's origin is
    // in top left corner, while chart's origin is in bottom left corner, hence we need to invert
    // the range on Y axis for the chart to display as though its origin is at bottom left.
    let y = ScaleLinear::new()
        .set_domain(vec![0.0, max])
        .set_range(vec![height - top - bottom, 0]);

    // Create VerticalBar view that is going to represent the data as vertical bars.
    let view = VerticalBarView::new()
        .set_x_scale(&x)
        .set_y_scale(&y)
        .set_custom_data_label(format!(
            "Total: {} {}",
            total, CONFIG.substrate.token_ticker
        ))
        .load_data(&data)
        .unwrap();

    let path = "/Users/user/Desktop/chart.svg";
    // Generate and save the chart.
    Chart::new()
        .set_width(width)
        .set_height(height)
        .set_margins(top, right, bottom, left)
        .add_title(String::from("Bar Chart"))
        .add_legend_at(AxisPosition::Top)
        .add_view(&view)
        .add_axis_bottom(&x)
        .add_axis_left(&y)
        .add_left_axis_label(format!("Reward ({})", CONFIG.substrate.token_ticker))
        //.add_bottom_axis_label("Categories")
        .set_bottom_axis_tick_label_rotation(45)
        .save(path)
        .unwrap();

    let mut opt = usvg::Options {
        resources_dir: std::fs::canonicalize(path)
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf())),
        ..Default::default()
    };
    opt.fontdb.load_system_fonts();

    let svg_data = std::fs::read(path).unwrap();
    let rtree = usvg::Tree::from_data(&svg_data, &opt.to_ref()).unwrap();

    let pixmap_size = rtree.svg_node().size.to_screen_size();
    let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();
    resvg::render(
        &rtree,
        usvg::FitTo::Original,
        tiny_skia::Transform::default(),
        pixmap.as_mut(),
    )
    .unwrap();
    pixmap
        .save_png("/Users/user/Desktop/Images/chart.png")
        .unwrap();
}
