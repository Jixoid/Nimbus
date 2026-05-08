/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


#define el else
#define ef else if

#include <android_native_app_glue.h>

#include "basis.hh"

#include "qcl/Application.hh"
#include "qcl/Control.hh"
#include "qcl/Window.hh"
#include "qcl/Graphic.hh"
#include "qcl/Platform.hh"

using namespace std;



struct nand_app_handle
{
    android_app *State = nil;
};

struct nand_win_handle
{
};

struct nand_sur_handle
{
};


struct nand_gl_handle
{
};


enum nand_msgtyp: u8
{
  amtMessage = 1,
  amtTask    = 2,
};

struct nand_message  // Max 20b  // using 17b
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

  nand_msgtyp Type; // 1b

  
  inline bool operator==(const nand_message &p0) const
  {
    if (Type != p0.Type)
      return false;


    switch (Type)
    {
      case amtMessage:
        return (
          Ctrl == p0.Ctrl &&
          Msg  == p0.Msg
        );

      case amtTask:
        return (
          Fun  == p0.Fun &&
          Data == p0.Data
        );
    
      default:
        #ifdef _DBG
        cerr << "qcl_error: Unknown message type" << endl;
        #endif
        return false;
    }

  }
};


namespace qcl::platform::window
{
  void New(handle *ret);


  #define Native ((nand_win_handle*)val)

  void Dis(handle  val);


  void Show(handle val);

  void Hide(handle val);


  void SizeSet(handle val, size_i32 Size);

  size_i32 SizeGet(handle val);


  void StartMove(handle val, poit_i32 Pos);

  #undef Native
}


namespace qcl::platform::surface
{
  void New(handle *ret, u32 nWidth, u32 nHeight);

  void New_Data(handle *ret, u32 nWidth, u32 nHeight, u32 *Data);

  void New_File_SVG(handle *ret, u32 nWidth, u32 nHeight, const char* Path);
  
  void New_Data_SVG(handle *ret, u32 nWidth, u32 nHeight, const u8* Data, u0 DSize);


  #define Native ((nand_sur_handle*)val)

  void Dis(handle  val);


  void Set_Color(handle  val, color Val);

  void Set_Source(handle  val, shared_ptr<qcl::surface> Val);

  void Set_SourceP(handle  val, qcl::surface *Val);

  void Set_FontSize(handle  val, f32 Size);

  void Set_LineSize(handle  val, f32 Size);


  void Set_Translate(handle  val, poit_f32 P);

  
  void Set_OP(handle  val, qcl::surface::operators  Op);
  
  qcl::surface::operators Get_OP(handle  val);


  void Set_Pos(handle  val, poit_f32 Val);

  void Set_rPos(handle  val, poit_f32 Val);

  void Set_Size(handle  val, u32 nWidth, u32 nHeight);


  void Set_Dirty(handle  val);

  void Get_Data(handle  val, u32 **Data);


  void Draw_Rect(handle  val, rect_f32 Area);

  void Draw_RectRound(handle  val, rect_f32 Area, f32 Radius);

  void Draw_RectRound4(handle  val, rect_f32 Area, f32 R_LT, f32 R_LB, f32 R_RT, f32 R_RB);

  void Draw_Line(handle  val, poit_f32 P1, poit_f32 P2);

  void Draw_ToLine(handle  val, poit_f32 P);

  void Draw_Arc(handle  val, poit_f32 O, f32 Radius, f32 Ang1, f32 Ang2);

  void Draw_Text(handle  val, const char* Text);


  void Calc_Text(handle  val, const char* Text, size_f32* Size);


  void Filter_GrayScale(handle  val, rect_f32 Area);
  

  void Connect(handle  val, handle Window);
  

  void Stroke(handle  val);

  void Fill(handle  val);

  void Paint(handle  val);

  void PaintA(handle  val, f64 A);

  void Clip(handle  val);
  
  void Clip_Reset(handle  val);


  void Sets_Push(handle  val);

  void Sets_Pop(handle  val);


  void Clear(handle  val);

  void Flush(handle  val);

  #undef Native
}


namespace qcl::platform::application
{
  void New(handle *ret)
  {
      auto *h = (nand_app_handle*)calloc(1, sizeof(nand_app_handle));
      *ret = (handle)h;
  }

  #define Native ((nand_app_handle*)val)

  void Dis(handle  val)
  {
      free(Native);
  }



  void PushMessage(handle val, qcl::control* Ctrl, controlMessages Msg);

  void PushTask(handle val, void (*Fun)(u0), u0 Data);


  bool CheckQueue(handle val, qcl::control* Ctrl, controlMessages Msg);


  void Run(handle val);

  #undef Native
}


namespace qcl::platform::opengl
{
  void New(handle *ret, u32 nWidth, u32 nHeight);

  #define Native ((nand_gl_handle*)val)

  void Dis(handle  val);


  void Set_Current(handle  val);

  void Set_Size(handle  val, u32 nWidth, u32 nHeight);


  void ReadBuffer(handle  val, u32 *PixBuf);

  void Flush(handle  val);

  #undef Native
}


namespace qcl::platform::dialog
{

  char* FileOpen(const char *Title, const char *Filters[]);  // with malloc

  char* FileSave(const char *Title, const char *Filters[]);  // with malloc

  void Message(const char *Text);

}

#undef __CurrentApp



[[gnu::weak]] void android_main(android_app *State)
{
  if (qcl::platform::qcl_entry1 != nil)
  {
    ohid App_H{};
    qcl::platform::application::New(&App_H);
    ((nand_app_handle*)App_H)->State = State;

    ohid Win_H{};
    qcl::platform::window::New(&Win_H);

    qcl::platform::qcl_entry1(App_H, Win_H);
  }

  ef (qcl::platform::qcl_entry2 != nil)
  {
    ohid App_H{};
    qcl::platform::application::New(&App_H);
    ((nand_app_handle*)App_H)->State = State;
    
    qcl::platform::qcl_entry2(App_H);
  }

  ef (qcl::platform::qcl_entry3 != nil)
  {
    qcl::platform::qcl_entry3();
  }

  el {
    #ifdef _DBG
    cerr << "Not found a qcl entry" << endl;
    #endif
  }
}

