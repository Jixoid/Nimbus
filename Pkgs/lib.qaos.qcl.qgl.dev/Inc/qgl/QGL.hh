/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/



#pragma once

#include "Basis.h"

#include "qcl/Control.hh"

using namespace std;
using namespace jix;
using namespace qcl;



namespace qgl
{
  void Register();


  class viewport: public qcl::control
  {
    public:
      viewport();
      ~viewport() override;

    public:
      handle GLContext;
      
    public:
      void SetCurrent();

      void Do_Resize() override;

      void Draw() override;
      void Draw_after() override;

  };

}
