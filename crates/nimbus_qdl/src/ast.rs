/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::collections::HashMap;


pub use nimbus_core::PropertyValue as QdlValue;

#[derive(Clone, Debug)]
pub struct QdlNode {
	pub kind: String,
	pub name: Option<String>,
	pub styles: Vec<String>,
	pub props: HashMap<String, QdlValue>,
	pub children: Vec<QdlNode>,
}

#[derive(Clone, Debug, Default)]
pub struct QdlDocument {
	pub imports: Vec<String>,
	pub styles: HashMap<String, HashMap<String, QdlValue>>,
	pub root: Option<QdlNode>,
}
