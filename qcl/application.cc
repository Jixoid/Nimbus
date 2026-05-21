/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


#include "qcl/basis.hh"

#include "qcl/window.hh"
#include "qcl/application.hh"
#include "qcl/platform.hh"
#include <algorithm>

using namespace std;



namespace qcl
{
  application::application()
    : m_ohid(platform::API->Application.new_())
  {}

  application::application(bool nMakeCurrent)
    : m_ohid(platform::API->Application.new_())
  {
    if (nMakeCurrent)
      makeCurrent();
  }

  application::application(handle nOHID)
  {
    m_ohid = nOHID;
  }

  application::application(handle nOHID, bool nMakeCurrent)
  {
    m_ohid = nOHID;

    if (nMakeCurrent)
      makeCurrent();
  }


  application::~application()
  {
    if (CurrentApp == this)
      CurrentApp = nil;
    
    platform::API->Application.dis_(ohid());
  }

  

  fun application::makeCurrent() -> void
  {
    CurrentApp = this;
  }


  fun application::regWindow(window* Win) -> void
  {
    auto it = std::find_if(m_winList.begin(), m_winList.end(), [&Win](pair<handle, window*> &X) -> bool
    {
      return X.second == Win;
    });


    if (it == m_winList.end())
      m_winList.push_back({Win->ohid(), Win});

    else
      *it = {Win->ohid(), Win};
  }

  fun application::delWindow(window* Win) -> void
  {
    auto it = std::find_if(m_winList.begin(), m_winList.end(), [&Win](pair<handle, window*> &X) -> bool
    {
      return X.second == Win;
    });

    if (it != m_winList.end())
      m_winList.erase(it);
  }


  fun application::pushMessage(visual* Ctrl, visual::messages Msg) -> void
  {
    platform::API->Application.pushMessage(ohid(), Ctrl, Msg);
  }

  fun application::pushTask(void (*Fun)(u0), u0 Data) -> void
  {
    platform::API->Application.pushTask(ohid(), Fun, Data);
  }


  fun application::checkQueue(visual* Ctrl, visual::messages Msg) -> bool
  {
    return platform::API->Application.checkQueue(ohid(), Ctrl, Msg);
  }


  fun application::run() -> void
  {
    platform::API->Application.run(ohid());
  }


  application *CurrentApp{};
}
