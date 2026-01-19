/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/



#include "Basis.h"

#include "qcl/Control.hh"
#include "qcl/Window.hh"
#include "qcl/Graphic.hh"
#include "qcl/Application.hh"
#include "qcl/Platform.hh"

using namespace std;
using namespace jix;



namespace qcl
{
  application::application()
  {
    platform::application::New(&OHID);
  }

  application::application(bool nMakeCurrent)
  {
    platform::application::New(&OHID);

    if (nMakeCurrent)
      MakeCurrent();
  }

  application::application(handle nOHID)
  {
    OHID = nOHID;
  }

  application::application(handle nOHID, bool nMakeCurrent)
  {
    OHID = nOHID;

    if (nMakeCurrent)
      MakeCurrent();
  }

  application::~application()
  {
    if (CurrentApp == this)
      CurrentApp = Nil;
    
    platform::application::Dis(OHID);
  }

  

  void application::MakeCurrent()
  {
    CurrentApp = this;
  }


  void application::Window_Reg(window* Win)
  {
    WinList[Win->OHID] = Win;
  }

  void application::Window_Dis(window* Win)
  {
    WinList.erase(Win->OHID);
  }


  void application::PushMessage(qcl::control* Ctrl, controlMessages Msg)
  {
    platform::application::PushMessage(OHID, Ctrl, Msg);
  }

  void application::PushTask(void (*Fun)(u0), u0 Data)
  {
    platform::application::PushTask(OHID, Fun, Data);
  }


  bool application::CheckQueue(qcl::control* Ctrl, controlMessages Msg)
  {
    return platform::application::CheckQueue(OHID, Ctrl, Msg);
  }


  void application::Run()
  {
    platform::application::Run(OHID);
  }


  application *CurrentApp = Nil;
}
