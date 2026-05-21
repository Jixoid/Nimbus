/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


#pragma once

#include "qstd/std_layout.hh"

using namespace qcl;



namespace qstd
{

  struct layout_vert: qstd::layout
  {
    fun doTiling(renderContext&) -> void override;
  };

}
