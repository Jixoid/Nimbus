/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


#pragma once

#include "qcl/basis.hh"
#include "qcl/dynamicLoad.hh"
#include "qstd/std_form.hh"
#include "qstd/std_layout.hh"
#include "qstd/std_layout_vert.hh"
#include "qstd/std_layout_horz.hh"
#include "qstd/std_layout_flow.hh"
#include "qstd/std_text.hh"
#include "qstd/std_icon.hh"
#include "qstd/std_edit.hh"
#include "qstd/std_radio.hh"
#include "qstd/std_check.hh"
#include "qstd/std_toggle.hh"
#include "qstd/std_progbar.hh"
#include "qstd/std_slider.hh"
#include "qstd/std_tabs.hh"
#include "qstd/std_choice.hh"
#include "qstd/std_grid.hh"
#include "qstd/std_popup_menu.hh"
#include "qstd/std_chip.hh"
#include "qstd/std_card.hh"
#include "qstd/std_multi_progbar.hh"
#include "qstd/std_interval.hh"
#include "qstd/std_viewport_vk.hh"



namespace qstd
{
  fun objs() -> qcl::dyn::objs;
}
