/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


#pragma once

#include "basis.hh"
#include "qcl/window.hh"

using namespace std;



namespace qcl
{

  /**
   * @brief Core application class managing the main event loop and windows.
   * 
   * The application class handles window registration, task queueing, message 
   * dispatching, and the main execution loop for a QCL application.
   */
  class application
  {
    friend class window;

    public:
      /**
       * @brief Default constructor.
       */
      application();

      /**
       * @brief Constructor that optionally makes the application current.
       * @param nMakeCurrent If true, sets this application as the current context.
       */
      explicit application(bool nMakeCurrent);

      /**
       * @brief Constructor initializing with an existing operating system handle.
       * @param nOHID The operating system handle ID.
       */
      explicit application(handle nOHID);

      /**
       * @brief Constructor with an operating system handle and current context flag.
       * @param nOHID The operating system handle ID.
       * @param nMakeCurrent If true, sets this application as the current context.
       */
      application(handle nOHID, bool nMakeCurrent);

      /**
       * @brief Destructor.
       */
      ~application();
  

    private:
      handle m_ohid{};

      vector<pair<handle, window*>> m_winList{};


    public:
      /**
       * @brief Gets the operating system handle ID.
       * @return The handle ID.
       */
      inline fun ohid() const noexcept -> handle {return m_ohid;}

      /**
       * @brief Gets the list of registered windows.
       * @return A constant reference to the vector of window pairs.
       */
      inline fun winList() const noexcept -> const vector<pair<handle, window*>>& {return m_winList;}


    protected:
      /**
       * @brief Registers a window with the application.
       * @param Win Pointer to the window to register.
       */
      fun regWindow(window* Win) -> void;

      /**
       * @brief Unregisters and deletes a window from the application.
       * @param Win Pointer to the window to delete.
       */
      fun delWindow(window* Win) -> void;
      

    public:
      /**
       * @brief Sets this application as the current active application context.
       */
      fun makeCurrent() -> void;
      
      /**
       * @brief Pushes a message to a visual control.
       * @param Ctrl Pointer to the target visual control.
       * @param Msg The message to push.
       */
      fun pushMessage(visual* Ctrl, visual::messages Msg) -> void;

      /**
       * @brief Pushes a task to the application's execution queue.
       * @param Fun Function pointer to the task.
       * @param Data Data to pass to the task function.
       */
      fun pushTask(void (*Fun)(u0), u0 Data) -> void;

      /**
       * @brief Checks the message queue for a specific visual control and message.
       * @param Ctrl Pointer to the target visual control.
       * @param Msg The message to check for.
       * @return True if the message is in the queue, false otherwise.
       */
      fun checkQueue(visual* Ctrl, visual::messages Msg) -> bool;
      
      /**
       * @brief Starts the main application execution loop.
       */
      fun run() -> void;
  };


  /**
   * @brief Pointer to the current active application.
   */
  extern application *CurrentApp;
}
