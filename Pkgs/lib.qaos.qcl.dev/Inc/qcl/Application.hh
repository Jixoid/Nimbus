/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/



#pragma once

#include <unordered_map>

#include "Basis.h"

#include "qcl/Control.hh"
#include "qcl/Graphic.hh"

using namespace std;
using namespace jix;



namespace qcl
{

  class application
  {
    friend class window;

    public:
      application();
      explicit application(bool nMakeCurrent);
      explicit application(handle nOHID);
      application(handle nOHID, bool nMakeCurrent);
      ~application();
  

    public:
      handle OHID;

      unordered_map<handle, qcl::window*> WinList;


    protected:
      void Window_Reg(qcl::window* Win);
      void Window_Dis(qcl::window* Win);
      
    public:
      void MakeCurrent();
      
      void PushMessage(qcl::control* Ctrl, controlMessages Msg);
      void PushTask(void (*Fun)(u0), u0 Data);

      bool CheckQueue(qcl::control* Ctrl, controlMessages Msg);
      
      void Run();
  };


  extern application *CurrentApp;
}
