/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/



#include <iterator>
#define el else
#define ef else if

#include <iostream>
#include <queue>

#include "Basis.h"

#include "qcl/Application.hh"
#include "qcl/Control.hh"
#include "qcl/Platform.hh"

#include <libloaderapi.h>
#include <minwindef.h>
#define byte win_byte
#include <windows.h>
#undef byte
#include <gdiplus.h>

using namespace std;


struct win_handle
{
  HWND hwnd;
};

struct graph_handle
{
  Gdiplus::Graphics* graphics;
  Gdiplus::Pen* currentPen;
  Gdiplus::SolidBrush* currentBrush;
  Gdiplus::Font* currentFont;
  Gdiplus::Bitmap* bitmap; // Off-screen çizimler için
  HWND               connectedHwnd;
  
  // GDI+ Resource management
  Gdiplus::Color     color;
  float              lineWidth;
};

struct app_handle
{
  HINSTANCE hInstance;       // Uygulama instance'ı
  bool      isRunning;       // Döngü kontrolü
  
  // Görev (Task) kuyruğu için yardımcı yapı
  struct TaskItem {
    void (*Fun)(void*);
    void* Data;
  };
  std::queue<TaskItem> taskQueue;
  std::mutex           queueMutex;
};

struct opengl_handle
{};



enum win_msgtyp: u8
{
  wmtMessage = 1,
  wmtTask    = 2,
};

struct win_message  // Max 20b  // using 17b
{
  union // 8b
  {
    qcl::control *Ctrl;
    void (*Fun)(u0);
  };

  union // 8b
  {
    qcl::controlMessages Msg;
    u0 Data;
  };

  win_msgtyp Type; // 1b

  
  inline bool operator==(const win_message &p0) const
  {
    if (Type != p0.Type)
      return false;


    switch (Type)
    {
      case wmtMessage:
        return (
          Ctrl == p0.Ctrl &&
          Msg  == p0.Msg
        );

      case wmtTask:
        return (
          Fun  == p0.Fun &&
          Data == p0.Data
        );
    
      default:
        cerr << "qcl_error: Unknown message type" << endl;
        return false;
    }

  }
};


#define __CurrentApp ((app_handle*)CurrentApp->OHID)




using namespace qcl;



namespace
{
  LRESULT CALLBACK DefaultWndProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam)
  {
    if (msg == WM_DESTROY) {
        PostQuitMessage(0);
        return 0;
    }
    return DefWindowProc(hwnd, msg, wParam, lParam);
  }
}


qcl::platform::api API_Win = {

  .Window = {
    #define Native ((win_handle*)val)

    .New = [](handle *ret)
    {
      // 1. Pencere Sınıfını Kaydet (Sadece bir kez yapılması yeterlidir)
      static bool classRegistered = false;
      const char* className = "QCL_Window_Class";

      if (!classRegistered) {
          WNDCLASSEX wc = {0};
          wc.cbSize = sizeof(WNDCLASSEX);
          wc.lpfnWndProc = DefaultWndProc;
          wc.hInstance = __CurrentApp->hInstance;
          wc.lpszClassName = className;
          wc.hCursor = LoadCursor(NULL, IDC_ARROW);
          wc.hbrBackground = (HBRUSH)(COLOR_WINDOW + 1);
          
          RegisterClassEx(&wc);
          classRegistered = true;
      }

      // 2. Pencereyi Oluştur
      HWND hwnd = CreateWindowEx(
        0, className, "QCL Window",
        WS_OVERLAPPEDWINDOW,
        CW_USEDEFAULT, CW_USEDEFAULT, 800, 600,
        NULL, NULL, __CurrentApp->hInstance, NULL
      );

      if (hwnd) {
        win_handle* win = new win_handle();
        win->hwnd = hwnd;
        *ret = (handle)win;
      }
    },

    .Dis = [](handle val)
    {
      DestroyWindow(Native->hwnd);
      delete Native;
    },

    .Show = [](handle val)
    {
      ShowWindow(Native->hwnd, SW_SHOW);
    },

    .Hide = [](handle val)
    {
      ShowWindow(Native->hwnd, SW_HIDE);
    },

    .SizeSet = [](handle val, size_i32 Size)
    {
      // SWP_NOMOVE kullanarak sadece boyutu değiştiririz
      SetWindowPos(Native->hwnd, NULL, 0, 0, Size.W, Size.H, SWP_NOMOVE | SWP_NOZORDER);
    },

    .SizeGet = [](handle val) -> size_i32
    {
      size_i32 s = {0, 0};
      if (val) {
          RECT rect;
          if (GetWindowRect(Native->hwnd, &rect))
          {
              s.W = rect.right - rect.left;
              s.H = rect.bottom - rect.top;
          }
      }
      return s;
    },

    .StartMove = [](handle val, poit_i32 Pos)
    {
      // SWP_NOSIZE kullanarak sadece konumu değiştiririz
      SetWindowPos(Native->hwnd, NULL, Pos.X, Pos.Y, 0, 0, SWP_NOSIZE | SWP_NOZORDER);
    },

    #undef Native
  },

  .Surface = {
    #define Native ((graph_handle*)val)
    
    .New = [](handle *ret, u32 nWidth, u32 nHeight)
    {
      auto* h = new graph_handle();
      h->bitmap = new Gdiplus::Bitmap(nWidth, nHeight, PixelFormat32bppARGB);
      h->graphics = Gdiplus::Graphics::FromImage(h->bitmap);
      h->currentPen = new Gdiplus::Pen(Gdiplus::Color(255, 0, 0, 0), 1.0f);
      h->currentBrush = new Gdiplus::SolidBrush(Gdiplus::Color(255, 0, 0, 0));
      *ret = (handle)h;
    },
    
    //void (*New_File_SVG)(handle *ret, u32 nWidth, u32 nHeight, const char* Path){};
    //void (*New_Data_SVG)(handle *ret, u32 nWidth, u32 nHeight, const u8* Data, u0 DSize){};

    .Dis = [](handle val)
    {
      delete Native->graphics;
      delete Native->currentPen;
      delete Native->currentBrush;
      if (Native->bitmap)
        delete Native->bitmap;
      delete Native;
    },

    

    .Set_Color = [](handle val, color Val)
    {
      // color formatınızın ARGB olduğunu varsayıyorum
      Gdiplus::Color c(Val.A, Val.R, Val.G, Val.B);
      Native->currentPen->SetColor(c);
      Native->currentBrush->SetColor(c);
    },

    //void (*Set_Source)(handle  val, qcl::surface *Val){};
    //void (*Set_FontSize)(handle  val, f32 Size){};

    .Set_LineSize = [](handle val, f32 Size)
    {
      Native->currentPen->SetWidth(Size);
    },

    //void (*Set_Translate)(handle  val, poit_f32 P){};

    //void (*Set_OP)(handle  val, qcl::surface::operators  Op){};
    //qcl::surface::operators (*Get_OP)(handle  val){};

    //void (*Set_Pos)(handle  val, poit_f32 Val){};
    //void (*Set_rPos)(handle  val, poit_f32 Val){};
    //void (*Set_Size)(handle  val, u32 nWidth, u32 nHeight){};

    //void (*Set_Dirty)(handle  val){};
    //void (*Get_Data)(handle  val, u32 **Data){};

    .Draw_Rect = [](handle val, rect_f32 Area)
    {
      Native->graphics->DrawRectangle(Native->currentPen, Area.X1, Area.Y1, Area.X2 -Area.X1, Area.Y2 -Area.Y1);
    },

    //void (*Draw_RectRound)(handle  val, rect_f32 Area, f32 Radius){};
    //void (*Draw_RectRound4)(handle  val, rect_f32 Area, f32 R_LT, f32 R_LB, f32 R_RT, f32 R_RB){};
    //void (*Draw_Line)(handle  val, poit_f32 P1, poit_f32 P2){};
    //void (*Draw_ToLine)(handle  val, poit_f32 P){};
    //void (*Draw_Arc)(handle  val, poit_f32 O, f32 Radius, f32 Ang1, f32 Ang2){};
    //void (*Draw_Text)(handle  val, const char* Text){};

    //void (*Calc_Text)(handle  val, const char* Text, size_f32 *Size){};

    //void (*Filter_GrayScale)(handle  val, rect_f32 Area){};

    .Connect = [](handle val, handle Window)
    {
      auto* win = (win_handle*)Window; // Önceki window_handle yapımız
      Native->connectedHwnd = win->hwnd;
      
      // Eğer doğrudan pencereye çizilecekse
      if (Native->graphics)
        delete Native->graphics;

      Native->graphics = Gdiplus::Graphics::FromHWND(win->hwnd);
    },

    //void (*Stroke)(handle  val){};
    
    .Fill = [](handle val)
    {
      // Genellikle QCL gibi kütüphanelerde Path oluşturulur, 
      // Fill dendiğinde o path boyanır. Şimdilik basitçe son çizimi doldurabiliriz.
    },

    //void (*Paint)(handle  val){};
    //void (*PaintA)(handle  val, f64 A){};
    //void (*Clip)(handle  val){};
    //void (*Clip_Reset)(handle  val){};

    //void (*Sets_Push)(handle  val){};
    //void (*Sets_Pop)(handle  val){};

    //void (*Clear)(handle  val){};
    //void (*Flush)(handle  val){};

    #undef Native
  },

  .Application = {
    #define Native ((app_handle*)val)

    .New = [](handle *ret)
    {
      app_handle* app = new app_handle();
      app->hInstance = GetModuleHandleA(NULL);
      app->isRunning = false;
      
      *ret = (handle)app;
    },

    .Dis = [](handle val)
    {
      delete Native;
    },

    .PushMessage = [](handle val, qcl::control* Ctrl, controlMessages Msg)
    {
      // WinAPI'de mesajlar genellikle HWND üzerinden gönderilir.
      // Ctrl nesnesinin içindeki win_handle (HWND) alınarak SendMessage çağrılır.
      //HWND target = (HWND)Ctrl->getNativeHandle(); 
      //SendMessage(target, (UINT)Msg, 0, 0);
    },

    .PushTask = [](handle val, void (*Fun)(u0), u0 Data)
    {
      Native->queueMutex.lock();
      Native->taskQueue.push({(void (*)(void*))Fun, (void*)Data});
      Native->queueMutex.unlock();

      // Mesaj döngüsünü uyandırmak için boş bir mesaj gönder
      PostThreadMessage(GetCurrentThreadId(), WM_NULL, 0, 0);
    },

    .CheckQueue = [](handle val, qcl::control* Ctrl, controlMessages Msg) -> bool
    {
      //MSG m;
      //HWND target = (HWND)Ctrl->getNativeHandle();
      //// Kuyrukta belirli bir mesajın olup olmadığını kontrol eder (PeekMessage)
      //return PeekMessage(&m, target, (UINT)Msg, (UINT)Msg, PM_NOREMOVE);
      return false;
    },

    .Run = [](handle val)
    {
      Native->isRunning = true;
      MSG msg;

      while (Native->isRunning && GetMessage(&msg, NULL, 0, 0))
      {
        TranslateMessage(&msg);
        DispatchMessage(&msg);

        // Her mesaj döngüsünde bekleyen görevleri (task) işle
        Native->queueMutex.lock();
        while (!Native->taskQueue.empty())
        {
          auto task = Native->taskQueue.front();
          Native->taskQueue.pop();
          Native->queueMutex.unlock();
          
          if (task.Fun) task.Fun(task.Data);
          
          Native->queueMutex.lock();
        }
        Native->queueMutex.unlock();
      }
    },

    #undef Native
  },

};

#undef __CurrentApp



int main()
{
  qcl::platform::API = &API_Win;


  HMODULE hMain = GetModuleHandle(NULL); 
  
  auto qcl_entry1 = (int (*)(ohid App_H, ohid Win_H)) GetProcAddress(hMain, "qcl_entry1");
  auto qcl_entry2 = (int (*)(ohid App_H))             GetProcAddress(hMain, "qcl_entry2");
  auto qcl_entry3 = (int (*)())                       GetProcAddress(hMain, "qcl_entry3");

  if (qcl_entry1)
  {
    ohid App_H{};
    qcl::platform::API->Application.New(&App_H);

    ohid Win_H{};
    qcl::platform::API->Window.New(&Win_H);
    
    return qcl_entry1(App_H, Win_H);
  }

  ef (qcl_entry2)
  {
    ohid App_H{};
    qcl::platform::API->Application.New(&App_H);
    
    return qcl_entry2(App_H);
  }

  ef (qcl_entry3)
  {
    return qcl_entry3();
  }

  el {
    #ifdef _DBG
    cerr << "Not found a qcl entry" << endl;
    #endif

    return -1;
  }
}

