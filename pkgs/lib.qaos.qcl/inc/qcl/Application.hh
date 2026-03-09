/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/



#pragma once

#include "Basis.h"
#include "qcl/Window.hh"

using namespace std;



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
  

    private:
      handle m_ohid{};

      vector<pair<handle, window*>> m_winList{};


    public:
      inline fun ohid() const noexcept -> handle {return m_ohid;}

      inline fun winList() const noexcept -> const vector<pair<handle, window*>>& {return m_winList;}


    protected:
      fun regWindow(window* Win) -> void;
      fun delWindow(window* Win) -> void;
      

    public:
      fun makeCurrent() -> void;
      
      fun pushMessage(visual* Ctrl, visual::messages Msg) -> void;
      fun pushTask(void (*Fun)(u0), u0 Data) -> void;

      fun checkQueue(visual* Ctrl, visual::messages Msg) -> bool;
      
      fun run() -> void;
  };


  extern application *CurrentApp;
}
