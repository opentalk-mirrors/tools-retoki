// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

mod component;
mod component_category;
mod component_category_identifier;
mod component_category_name;
mod component_identifier;
mod component_name;
mod component_release;
mod product_name;
mod release;
mod release_series;
mod releases;
mod series_codename;
mod series_number;

pub use component::Component;
pub use component_category::ComponentCategory;
pub use component_category_identifier::ComponentCategoryIdentifier;
pub use component_category_name::ComponentCategoryName;
pub use component_identifier::ComponentIdentifier;
pub use component_name::ComponentName;
pub use component_release::ComponentRelease;
pub use product_name::ProductName;
pub use release::Release;
pub use release_series::ReleaseSeries;
pub use releases::Releases;
pub use series_codename::SeriesCodename;
pub use series_number::SeriesNumber;
