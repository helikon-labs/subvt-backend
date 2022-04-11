//! # rustplotlib
//!
//! A visualization library for Rust inspired by D3.js.
//!
//! ## Features
//!
//! This is a WIP, but so far the library supports the following chart types:
//!
//! 1. Bar Chart (horizontal and vertical)
//! 2. Stacked Bar Chart (horizontal and vertical)
//!
//! ## Abstraction Layers
//!
//! There are several abstractions at the foundation of this visualization library:
//!
//!   Page
//!   └- Grid
//!      └- Chart
//!         ├- Axes
//!         └- View
//!            └- Dataset
//!
#![allow(unused)]

pub(crate) use crate::plotlib::axis::{Axis, AxisPosition};
pub(crate) use crate::plotlib::chart::Chart;
pub(crate) use crate::plotlib::colors::Color;
pub(crate) use crate::plotlib::components::bar::BarLabelPosition;
pub(crate) use crate::plotlib::components::line::LineSeries;
pub(crate) use crate::plotlib::components::scatter::{MarkerType, PointLabelPosition};
pub(crate) use crate::plotlib::scales::band::ScaleBand;
pub(crate) use crate::plotlib::scales::linear::ScaleLinear;
pub(crate) use crate::plotlib::scales::Scale;
pub(crate) use crate::plotlib::views::area::AreaSeriesView;
pub(crate) use crate::plotlib::views::datum::{BarDatum, PointDatum};
pub(crate) use crate::plotlib::views::horizontal_bar::HorizontalBarView;
pub(crate) use crate::plotlib::views::line::LineSeriesView;
pub(crate) use crate::plotlib::views::scatter::ScatterView;
pub(crate) use crate::plotlib::views::vertical_bar::VerticalBarView;

mod axis;
mod chart;
mod colors;
mod components;
mod legend;
mod scales;
// mod view;
mod views;
