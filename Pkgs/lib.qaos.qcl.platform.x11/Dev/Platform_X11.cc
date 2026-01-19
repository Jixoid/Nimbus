/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/



#define el else
#define ef else if

#include <iostream>
#include <memory>
#include <cmath>

extern "C"
{
  #include <X11/X.h>
  #include <X11/Xlib.h>
  #include <X11/Xutil.h>
  #include <X11/Xatom.h>
  #include <stdlib.h>
  
  #include <cairo/cairo.h>
  #include <cairo/cairo-xlib.h>
  
  #include <memory.h>
  #include <unistd.h>

  #include <GL/glew.h>
  #include <GL/gl.h>
  #include <GL/glx.h>
  #include <GL/glext.h>
}

#include <resvg.h>

#include "Basis.h"

#include "qcl/Application.hh"
#include "qcl/Control.hh"
#include "qcl/Window.hh"
#include "qcl/Graphic.hh"
#include "qcl/Platform.hh"

using namespace std;
using namespace jix;


struct x11_handle
{
  Window   window;
  int      screen;
  GC       gc;
  int      width, height;
  int      posX, posY;

  Pixmap Buf1 = 0, Buf2 = 0;

  XIC xic;
};

struct cairo_handle
{
  cairo_surface_t *Sur;
  cairo_t *Car;

  cairo_surface_t *Sur2 = Nil;
  cairo_t *Car2 = Nil;
  
  x11_handle *QWin;
  
  void *WillDestroy = Nil;
};



struct app_handle
{
  Display *dpy;
  Window   win;
  
  XIM xim;

  Atom qcl_ipc;
  Atom WMProtocol;
  Atom WDelete;
};

struct opengl_handle
{
  u32 Width, Height;

  GLXFBConfig *FBConfigs;
  GLXContext Context;
  GLXPbuffer PBuffer;
  GLuint FBO, Tex;
};


enum x11_msgtyp: u8
{
  xmtMessage = 1,
  xmtTask    = 2,
};

struct x11_message  // Max 20b  // using 17b
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

  x11_msgtyp Type; // 1b

  
  inline bool operator==(const x11_message &p0) const
  {
    if (Type != p0.Type)
      return false;


    switch (Type)
    {
      case xmtMessage:
        return (
          Ctrl == p0.Ctrl &&
          Msg  == p0.Msg
        );

      case xmtTask:
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


#define XSelect (ExposureMask | ButtonPressMask  | ButtonReleaseMask | KeyPressMask | KeyReleaseMask | PointerMotionMask | StructureNotifyMask | PropertyChangeMask)

#define __CurrentApp ((app_handle*)CurrentApp->OHID)


namespace qcl::platform::window
{
  void New(handle *ret)
  {
    x11_handle* h = (x11_handle*)calloc(1, sizeof(x11_handle));
    *ret = (handle)h;

    h->screen = DefaultScreen(__CurrentApp->dpy);
    h->width = 800;
    h->height = 600;

    XSetWindowAttributes attrs;
    attrs.background_pixmap = None;
    attrs.bit_gravity = StaticGravity;
    attrs.event_mask = XSelect;

    h->window = XCreateWindow(
      __CurrentApp->dpy,
      RootWindow(__CurrentApp->dpy, h->screen),
      10, 10,
      h->width, h->height,
      1,
      CopyFromParent,
      InputOutput,
      CopyFromParent,
      CWBackPixmap | CWEventMask | CWBitGravity,
      &attrs
    );


    h->gc = XCreateGC(__CurrentApp->dpy, h->window, 0, NULL);


    h->Buf1 = XCreatePixmap(
      __CurrentApp->dpy,
      h->window,
      h->width,
      h->height,
      DefaultDepth(__CurrentApp->dpy, h->screen)
    );

    h->Buf2 = XCreatePixmap(
      __CurrentApp->dpy,
      h->window,
      h->width,
      h->height,
      DefaultDepth(__CurrentApp->dpy, h->screen)
    );


    h->xic = Nil;
    if (__CurrentApp->xim)
    {
      h->xic = XCreateIC(
        __CurrentApp->xim,
        XNInputStyle, XIMPreeditNothing | XIMStatusNothing,
        XNClientWindow, h->window,
        Nil
      );
    }


    XSetWMProtocols(__CurrentApp->dpy, h->window, &__CurrentApp->WDelete, 1);
  }


  #define Native ((x11_handle*)val)

  void Dis(handle  val)
  {
    XFreePixmap(__CurrentApp->dpy, Native->Buf1);
    XFreePixmap(__CurrentApp->dpy, Native->Buf2);

    XFreeGC(__CurrentApp->dpy, Native->gc);
    XDestroyWindow(__CurrentApp->dpy, Native->window);

    free(Native);
  }


  void Show(handle val)
  {
    XMapWindow(__CurrentApp->dpy, Native->window);
    XFlush(__CurrentApp->dpy);
  }

  void Hide(handle val)
  {
    XUnmapWindow(__CurrentApp->dpy, Native->window);
    XFlush(__CurrentApp->dpy);
  }


  void SizeSet(handle val, size_i32 Size)
  {
    XResizeWindow(__CurrentApp->dpy, Native->window, Size.W, Size.H);
    XFlush(__CurrentApp->dpy);
    Native->width  = Size.W;
    Native->height = Size.H;
  }

  size_i32 SizeGet(handle val)
  {
    return {
      Native->width,
      Native->height
    };
  }


  void StartMove(handle val, poit_i32 Pos)
  {
    XEvent ev;
    Atom moveresize = XInternAtom(__CurrentApp->dpy, "_NET_WM_MOVERESIZE", False);

    memset(&ev, 0, sizeof(ev));
    ev.xclient.type = ClientMessage;
    ev.xclient.send_event = True;
    ev.xclient.message_type = moveresize;
    ev.xclient.window = Native->window;
    ev.xclient.format = 32;

    // root_x, root_y = fare konumu
    ev.xclient.data.l[0] = Pos.X +Native->posX;
    ev.xclient.data.l[1] = Pos.Y +Native->posY;
    ev.xclient.data.l[2] = 8; // 8 = hareket (move)
    ev.xclient.data.l[3] = 1; // mouse button 1
    ev.xclient.data.l[4] = 0;

    XSendEvent(
      __CurrentApp->dpy,
      DefaultRootWindow(__CurrentApp->dpy),
      False,
      SubstructureRedirectMask | SubstructureNotifyMask,
      &ev
    );
    XFlush(__CurrentApp->dpy);
  }

  #undef Native
}


namespace qcl::platform::surface
{
  void New(handle *ret, u32 nWidth, u32 nHeight)
  {
    cairo_handle *h = (cairo_handle*)calloc(1, sizeof(cairo_handle));
    *ret = (handle)h;


    h->Sur = cairo_image_surface_create(CAIRO_FORMAT_ARGB32, nWidth, nHeight);
    h->Car = cairo_create(h->Sur);
  }

  void New_Data(handle *ret, u32 nWidth, u32 nHeight, u32 *Data)
  {
    cairo_handle *h = (cairo_handle*)calloc(1, sizeof(cairo_handle));
    *ret = (handle)h;

    h->Sur = cairo_image_surface_create_for_data(
      (unsigned char*)Data,
      CAIRO_FORMAT_ARGB32,
      nWidth, nHeight,
      nWidth*sizeof(u32)
    );
    h->Car = cairo_create(h->Sur);
  }

  void New_File_SVG(handle *ret, u32 nWidth, u32 nHeight, const char* Path)
  {
    resvg_options *opt = resvg_options_create();
    resvg_options_load_system_fonts(opt);
    resvg_options_set_shape_rendering_mode(opt, RESVG_SHAPE_RENDERING_GEOMETRIC_PRECISION);

    resvg_render_tree *tree = Nil;
    
    if (RESVG_OK != resvg_parse_tree_from_file(Path, opt, &tree))
    {
      qcl_error(string("SVG can not parsed!: "+string(Path)).c_str());
      ret = Nil;
      return;
    }

    resvg_size size = resvg_get_image_size(tree);
    resvg_transform transform = resvg_transform_identity();
    transform.a = (f64)nWidth / size.width;
    transform.d = (f64)nHeight / size.height;


    char *buffer = (char*)calloc(1, nWidth *nHeight *4);

    resvg_render(tree, transform, nWidth, nHeight, buffer);

    for (u64 i = 0; i < nWidth *nHeight; i++)
    {
      struct __y {u8 R,G,B,A;} *T = (__y*)&buffer[i*4];

      // BGRA -> RGBA
      u8 Temp = T->R;
      T->R = T->B;
      T->B = Temp;
    }

    handle H;
    New_Data(&H, nWidth, nHeight, (u32*)buffer);
    *ret = H;

    ((cairo_handle*)H)->WillDestroy = (void*)buffer;

    resvg_tree_destroy(tree);
    resvg_options_destroy(opt);
  }
  
  void New_Data_SVG(handle *ret, u32 nWidth, u32 nHeight, const u8* Data, u0 DSize)
  {
    // Render opsiyonları oluştur
    resvg_options *opt = resvg_options_create();
    resvg_options_load_system_fonts(opt);
    resvg_options_set_shape_rendering_mode(opt, RESVG_SHAPE_RENDERING_GEOMETRIC_PRECISION);

    resvg_render_tree *tree = Nil;
    
    if (RESVG_OK != resvg_parse_tree_from_data((const char*)Data, DSize, opt, &tree))
    {
      qcl_error(string("SVG can not parsed!").c_str());
      ret = Nil;
      return;
    }

    resvg_size size = resvg_get_image_size(tree);
    resvg_transform transform = resvg_transform_identity();
    transform.a = (f64)nWidth / size.width;
    transform.d = (f64)nHeight / size.height;

    char *buffer = (char*)malloc(nWidth *nHeight *4); // RGBA buffer

    resvg_render(tree, transform, nWidth, nHeight, buffer);
    
    for (u64 i = 0; i < nWidth *nHeight; i++)
    {
      struct __y {u8 R,G,B,A;} *T = (__y*)&buffer[i*4];

      // BGRA -> RGBA
      u8 Temp = T->R;
      T->R = T->B;
      T->B = Temp;
    }

    handle H;
    New_Data(&H, nWidth, nHeight, (u32*)buffer);
    *ret = H;

    ((cairo_handle*)H)->WillDestroy = (void*)buffer;

    resvg_tree_destroy(tree);
    resvg_options_destroy(opt);
  }


  #define Native ((cairo_handle*)val)

  void Dis(handle  val)
  {
    cairo_destroy(Native->Car);
    cairo_surface_destroy(Native->Sur);

    if (Native->WillDestroy != Nil)
      free(Native->WillDestroy);

    free(Native);
  }


  void Set_Color(handle  val, color Val)
  {
    cairo_set_source_rgba(Native->Car, Val.R, Val.G, Val.B, Val.A);
  }

  void Set_Source(handle  val, qcl::surface *Val)
  {
    f64 X,Y;
    cairo_get_current_point(Native->Car, &X, &Y);

    cairo_set_source_surface(Native->Car, ((cairo_handle*)Val->OHID)->Sur, X,Y);
  }

  void Set_FontSize(handle  val, f32 Size)
  {
    cairo_set_font_size(Native->Car, Size);
  }

  void Set_LineSize(handle  val, f32 Size)
  {
    cairo_set_line_width(Native->Car, Size);
  }


  void Set_Translate(handle  val, poit_f32 P)
  {
    cairo_translate(Native->Car, P.X, P.Y);
  }

  
  void Set_OP(handle  val, qcl::surface::operators  Op)
  {
    cairo_operator_t C_Op;

    switch (Op)
    {
      case qcl::surface::operators::soNormal:
      {
        C_Op = CAIRO_OPERATOR_OVER;
        break;
      }

      case qcl::surface::operators::soClear:
      {
        C_Op = CAIRO_OPERATOR_CLEAR;
        break;
      }

      case qcl::surface::operators::soIn:
      {
        C_Op = CAIRO_OPERATOR_IN;
        break;
      }

      default:
      {
        C_Op = CAIRO_OPERATOR_OVER;
        break;
      }

    }


    cairo_set_operator(Native->Car, C_Op);
  }
  
  qcl::surface::operators Get_OP(handle  val)
  {
    auto C_Op = cairo_get_operator(Native->Car);
    
    switch (C_Op)
    {
      case CAIRO_OPERATOR_OVER:  return qcl::surface::operators::soNormal;
      case CAIRO_OPERATOR_CLEAR: return qcl::surface::operators::soClear;
      case CAIRO_OPERATOR_IN:    return qcl::surface::operators::soIn;
      
      default: return qcl::surface::operators::soNormal;
    }
  }


  void Set_Pos(handle  val, poit_f32 Val)
  {
    cairo_move_to(Native->Car, Val.X, Val.Y);
  }

  void Set_rPos(handle  val, poit_f32 Val)
  {
    cairo_rel_move_to(Native->Car, Val.X, Val.Y);
  }

  void Set_Size(handle  val, u32 nWidth, u32 nHeight)
  {
    cairo_destroy(Native->Car);
    cairo_surface_destroy(Native->Sur);


    if (Native->QWin == Nil)
    {
      Native->Sur = cairo_image_surface_create(CAIRO_FORMAT_ARGB32, nWidth, nHeight);
      Native->Car = cairo_create(Native->Sur);
    }

    else
    {
      cairo_destroy(Native->Car2);
      cairo_surface_destroy(Native->Sur2);
    
      Native->Sur = cairo_xlib_surface_create(
        __CurrentApp->dpy,
        Native->QWin->Buf1,
        DefaultVisual(
          __CurrentApp->dpy,
          DefaultScreen(__CurrentApp->dpy)
        ),
        Native->QWin->width,
        Native->QWin->height
      );
      Native->Car = cairo_create(Native->Sur);


      Native->Sur2 = cairo_xlib_surface_create(
        __CurrentApp->dpy,
        Native->QWin->Buf2,
        DefaultVisual(
          __CurrentApp->dpy,
          DefaultScreen(__CurrentApp->dpy)
        ),
        Native->QWin->width,
        Native->QWin->height
      );
      Native->Car2 = cairo_create(Native->Sur2);
    }
  }


  void Set_Dirty(handle  val)
  {
    cairo_surface_mark_dirty(Native->Sur);
  }

  void Get_Data(handle  val, u32 **Data)
  {
    *Data = (u32*)cairo_image_surface_get_data(Native->Sur);
  }


  void Draw_Rect(handle  val, rect_f32 Area)
  {
    cairo_rectangle(Native->Car, Area.X1, Area.Y1, Area.X2 -Area.X1, Area.Y2 -Area.Y1);
  }

  void Draw_RectRound(handle  val, rect_f32 Area, f32 Radius)
  {
    if (Radius > (Area.Y2-Area.Y1) /2)
      Radius = (Area.Y2-Area.Y1) /2;

    if (Radius > (Area.X2-Area.X1) /2)
      Radius = (Area.X2-Area.X1) /2;


    //cairo_new_path(Native->Car);

    cairo_arc(Native->Car, Area.X1 +Radius, Area.Y1 +Radius, Radius, M_PI, 3 * M_PI / 2);
    cairo_line_to(Native->Car, Area.X2 -Radius, Area.Y1);
    cairo_arc(Native->Car, Area.X2 -Radius, Area.Y1 +Radius, Radius, 3 * M_PI / 2, 0);
    cairo_line_to(Native->Car, Area.X2, Area.Y2 -Radius);
    cairo_arc(Native->Car, Area.X2 -Radius, Area.Y2 -Radius, Radius, 0, M_PI / 2);
    cairo_line_to(Native->Car, Area.X1 +Radius, Area.Y2);
    cairo_arc(Native->Car, Area.X1 +Radius, Area.Y2 -Radius, Radius, M_PI / 2, M_PI);
    cairo_line_to(Native->Car, Area.X1, Area.Y1 +Radius);

    //cairo_close_path(Native->Car);
  }

  void Draw_RectRound4(handle  val, rect_f32 Area, f32 R_LT, f32 R_LB, f32 R_RT, f32 R_RB)
  {
    f32 W = (Area.X2 -Area.X1);
    f32 H = (Area.Y2 -Area.Y1);

    f32 scale = 1.0f;

    // Dikey limit
    if (R_LT +R_LB > H) 
      scale = min(scale, H /(R_LT +R_LB));
    if (R_RT +R_RB > H) 
      scale = min(scale, H /(R_RT +R_RB));

    // Yatay limit
    if (R_LT +R_RT > W) 
      scale = min(scale, W /(R_LT +R_RT));
    if (R_LB +R_RB > W) 
      scale = min(scale, W /(R_LB +R_RB));


    R_LT *= scale;
    R_LB *= scale;
    R_RT *= scale;
    R_RB *= scale;


    //cairo_new_path(Native->Car);

    cairo_arc(Native->Car, Area.X1 +R_LT, Area.Y1 +R_LT, R_LT, M_PI, 3 * M_PI / 2);
    cairo_line_to(Native->Car, Area.X2 -R_RT, Area.Y1);
    cairo_arc(Native->Car, Area.X2 -R_RT, Area.Y1 +R_RT, R_RT, 3 * M_PI / 2, 0);
    cairo_line_to(Native->Car, Area.X2, Area.Y2 -R_RB);
    cairo_arc(Native->Car, Area.X2 -R_RB, Area.Y2 -R_RB, R_RB, 0, M_PI / 2);
    cairo_line_to(Native->Car, Area.X1 +R_LB, Area.Y2);
    cairo_arc(Native->Car, Area.X1 +R_LB, Area.Y2 -R_LB, R_LB, M_PI / 2, M_PI);
    cairo_line_to(Native->Car, Area.X1, Area.Y1 +R_LT);

    //cairo_close_path(Native->Car);
  }

  void Draw_Line(handle  val, poit_f32 P1, poit_f32 P2)
  {
    cairo_move_to(Native->Car, P1.X, P1.Y);

    cairo_line_to(Native->Car, P2.X, P2.Y);
  }

  void Draw_ToLine(handle  val, poit_f32 P)
  {
    cairo_line_to(Native->Car, P.X, P.Y);
  }

  void Draw_Arc(handle  val, poit_f32 O, f32 Radius, f32 Ang1, f32 Ang2)
  {
    cairo_arc(Native->Car, O.X, O.Y, Radius, Ang1, Ang2);
  }

  void Draw_Text(handle  val, const char* Text)
  {
    cairo_font_extents_t Fx;
    cairo_font_extents(Native->Car, &Fx);

    cairo_rel_move_to(Native->Car, 0, Fx.ascent -Fx.descent);
    cairo_text_path(Native->Car, Text);
  }


  void Calc_Text(handle  val, const char* Text, size_f32* Size)
  {
    cairo_text_extents_t Ex;

    cairo_text_extents(Native->Car, Text, &Ex);


    *Size = {(f32)Ex.width, (f32)-Ex.y_bearing};
  }


  void Filter_GrayScale(handle  val, rect_f32 Area)
  {
    int width = cairo_image_surface_get_width(Native->Sur);
    int height = cairo_image_surface_get_height(Native->Sur);
    unsigned char* data = cairo_image_surface_get_data(Native->Sur);
    int stride = cairo_image_surface_get_stride(Native->Sur);
    cairo_format_t fmt = cairo_image_surface_get_format(Native->Sur);

    cairo_surface_flush(Native->Sur);

    i32 x1 = std::max(0, (i32)Area.X1);
    i32 y1 = std::max(0, (i32)Area.Y1);
    i32 x2 = std::min(width, (i32)Area.X2);
    i32 y2 = std::min(height, (i32)Area.Y2);

    for (int y = y1; y < y2; ++y)
      for (int x = x1; x < x2; ++x) {
        unsigned char* src = data + y * stride + x * 4;

        // ARGB: [B, G, R, A] in Cairo image surfaces (little endian)
        uint8_t Bc = src[0];
        uint8_t Gc = src[1];
        uint8_t Rc = src[2];
        uint8_t Ac = src[3];

        uint8_t gray = (uint8_t)(0.299 * Rc + 0.587 * Gc + 0.114 * Bc);
        src[0] = gray;
        src[1] = gray;
        src[2] = gray;
        src[3] = Ac;  // alpha'yı bozmadan taşı
      }

    cairo_surface_mark_dirty(Native->Sur);
  }
  

  void Connect(handle  val, handle Window)
  {
    cairo_destroy(Native->Car);
    cairo_surface_destroy(Native->Sur);



    Native->QWin = (x11_handle*)Window;

    Native->Sur = cairo_xlib_surface_create(
      __CurrentApp->dpy,
      Native->QWin->Buf1,
      DefaultVisual(
        __CurrentApp->dpy,
        DefaultScreen(__CurrentApp->dpy)
      ),
      Native->QWin->width,
      Native->QWin->height
    );
    Native->Car = cairo_create(Native->Sur);

    Native->Sur2 = cairo_xlib_surface_create(
      __CurrentApp->dpy,
      Native->QWin->Buf2,
      DefaultVisual(
        __CurrentApp->dpy,
        DefaultScreen(__CurrentApp->dpy)
      ),
      Native->QWin->width,
      Native->QWin->height
    );
    Native->Car2 = cairo_create(Native->Sur2);
  }
  

  void Stroke(handle  val)
  {
    cairo_stroke(Native->Car);
  }

  void Fill(handle  val)
  {
    cairo_fill(Native->Car);
  }

  void Paint(handle  val)
  {
    cairo_paint(Native->Car);
  }

  void PaintA(handle  val, f64 A)
  {
    cairo_paint_with_alpha(Native->Car, A);
  }

  void Clip(handle  val)
  {
    cairo_clip(Native->Car);
  }
  
  void Clip_Reset(handle  val)
  {
    cairo_reset_clip(Native->Car);
  }


  void Sets_Push(handle  val)
  {
    cairo_save(Native->Car);
  }

  void Sets_Pop(handle  val)
  {
    cairo_restore(Native->Car);
  }


  void Clear(handle  val)
  {
    auto bak = cairo_get_operator(Native->Car);

    auto width  = cairo_image_surface_get_width(Native->Sur);
    auto height = cairo_image_surface_get_height(Native->Sur);


    cairo_set_operator(Native->Car, CAIRO_OPERATOR_SOURCE);
    cairo_set_source_rgba(Native->Car, 0,0,0,0);
    cairo_rectangle(Native->Car, 0,0, width, height);
    cairo_fill(Native->Car);


    cairo_set_operator(Native->Car, bak);
  }

  void Flush(handle  val)
  {
    cairo_surface_flush(Native->Sur);
    XFlush(__CurrentApp->dpy);


    if (Native->QWin != Nil)
    {
      XCopyArea(
        __CurrentApp->dpy, 
        Native->QWin->Buf1, Native->QWin->window, Native->QWin->gc,
        0,0,
        Native->QWin->width, Native->QWin->height,
        0,0
      );

      XFlush(__CurrentApp->dpy);


      std::swap(Native->Sur, Native->Sur2);
      std::swap(Native->Car, Native->Car2);
      std::swap(Native->QWin->Buf1, Native->QWin->Buf2);
    }
  }

  #undef Native
}


namespace qcl::platform::application
{
  void New(handle *ret)
  {
    app_handle* h = (app_handle*)calloc(1, sizeof(app_handle));
    *ret = (handle)h;

    h->dpy = XOpenDisplay(NULL);

    h->win = XCreateSimpleWindow(
      h->dpy,
      RootWindow(h->dpy, DefaultScreen(h->dpy)),
      10, 10, 10, 10,
      1,
      BlackPixel(h->dpy, DefaultScreen(h->dpy)),
      WhitePixel(h->dpy, DefaultScreen(h->dpy))
    );
    XSelectInput(h->dpy, h->win, SubstructureNotifyMask | StructureNotifyMask );


    h->xim = XOpenIM(h->dpy, NULL, NULL, NULL);
    
    h->qcl_ipc = XInternAtom(h->dpy, "QCL_IPC", False);

    h->WMProtocol = XInternAtom(h->dpy, "WM_PROTOCOLS", False);
    h->WDelete = XInternAtom(h->dpy, "WM_DELETE_WINDOW", False);
  }

  #define Native ((app_handle*)val)

  void Dis(handle  val)
  {
    XDestroyWindow(Native->dpy, Native->win);

    XCloseDisplay(Native->dpy);

    free(Native);
  }



  void PushMessage(handle val, qcl::control* Ctrl, controlMessages Msg)
  {
    XEvent Ev = {};
    
    Ev.type = ClientMessage;
    Ev.xclient.message_type = __CurrentApp->qcl_ipc;
    Ev.xclient.display = Native->dpy;
    Ev.xclient.window = DefaultRootWindow(Native->dpy);
    Ev.xclient.format = 32;

    x11_message *XMsg = new x11_message();
    *XMsg = {
      .Ctrl = Ctrl,
      .Msg  = Msg,
      .Type = xmtMessage,
    };

    Ev.xclient.data.l[0] = (long)((u0)XMsg & 0xffffffff);
    Ev.xclient.data.l[1] = (long)(((u0)XMsg >> 32) & 0xffffffff);


    XSendEvent(Native->dpy, Native->win, False, NoEventMask, &Ev);
    XFlush(Native->dpy);
  }

  void PushTask(handle val, void (*Fun)(u0), u0 Data)
  {
    XEvent Ev = {};
    
    Ev.type = ClientMessage;
    Ev.xclient.message_type = __CurrentApp->qcl_ipc;
    Ev.xclient.display = Native->dpy;
    Ev.xclient.window = DefaultRootWindow(Native->dpy);
    Ev.xclient.format = 32;

    x11_message *XMsg = new x11_message();
    *XMsg = {
      .Fun  = Fun,
      .Data = Data,
      .Type = xmtTask,
    };

    Ev.xclient.data.l[0] = (long)((u0)XMsg & 0xffffffff);
    Ev.xclient.data.l[1] = (long)(((u0)XMsg >> 32) & 0xffffffff);
    

    XSendEvent(Native->dpy, Native->win, False, NoEventMask, &Ev);
    XFlush(Native->dpy);
  }


  shiftStateSet XStateToQShift(unsigned int XState)
  {
    shiftStateSet Ret = 0;

    if (XState & Button1Mask) Ret |= shiftStates::ssLeft;
    if (XState & Button2Mask) Ret |= shiftStates::ssMiddle;
    if (XState & Button3Mask) Ret |= shiftStates::ssRight;

    if (XState & ControlMask) Ret |= shiftStates::ssCtrl;
    if (XState & ShiftMask)   Ret |= shiftStates::ssShift;
    if (XState & Mod1Mask)    Ret |= shiftStates::ssAlt;
    if (XState & Mod4Mask)    Ret |= shiftStates::ssSuper;
    if (XState & Mod5Mask)    Ret |= shiftStates::ssAltGr;

    return Ret;
  };


  Bool __CheckDuplicate(Display *dpy, XEvent *Ev, XPointer Arg)
  {
    // arg = kontrol pointer'ı
    x11_message *LXMsg = (x11_message*)Arg;

    if (Ev->type != ClientMessage)
      return False;


    x11_message *XMsg = (x11_message*)&Ev->xclient.data;

    if (*LXMsg == *XMsg)
      return True;

    return False;
  }


  bool __CheckWEvent(Display *dpy, qcl::window *QWin, int MsgType)
  {   
    XEvent Ev;
    struct __wevent
    {
      Window Win;
      int    Typ;
      bool   Ret;
    } WEv = {
      .Win = ((x11_handle*)QWin->OHID)->window,
      .Typ = MsgType,
      .Ret = false,
    };


    XCheckIfEvent(dpy, &Ev, 
      [](Display *dpy, XEvent *Ev, XPointer Arg) -> Bool
      {
        // arg = kontrol pointer'ı
        __wevent *WEv = (__wevent*)Arg;

        if (Ev->type == MotionNotify && (Ev->xmotion.state & (Button1Mask | Button2Mask | Button3Mask)) == 0)
          return false;

        WEv->Ret |= (
          Ev->xany.window == WEv->Win &&
          Ev->type == WEv->Typ
        );

        return false;
      },
      (XPointer)&WEv
    );

    return WEv.Ret;
  }



  bool CheckQueue(handle val, qcl::control* Ctrl, controlMessages Msg)
  {
    switch (Msg)
    {
      case cmPaint:
      {
        x11_message XMsg = {
          Ctrl, Msg,
          xmtMessage,
        };

        XEvent Cac;
        if (XPeekIfEvent(Native->dpy, &Cac, &__CheckDuplicate, (XPointer)&XMsg))
          return true;
        

        if (auto C = dynamic_cast<qcl::window*>(Ctrl); C != Nil)
          return __CheckWEvent(Native->dpy, C, Expose);
      }

      case cmMouseDown:
      {
        if (auto C = dynamic_cast<qcl::window*>(Ctrl); C != Nil)
          return __CheckWEvent(Native->dpy, C, ButtonPress);
      }

      case cmMouseUp:
      {
        if (auto C = dynamic_cast<qcl::window*>(Ctrl); C != Nil)
          return __CheckWEvent(Native->dpy, C, ButtonRelease);
      }

      case cmMouseMove:
      {
        if (auto C = dynamic_cast<qcl::window*>(Ctrl); C != Nil)
          return __CheckWEvent(Native->dpy, C, MotionNotify);
      }

      default:
        return false;
    }
  }


  qcl::window* __FindWinFromWID(Window WID)
  {
    for (auto &[XKey, Win]: CurrentApp->WinList)
      if (((x11_handle*)XKey)->window == WID)
        return Win;

    return Nil;
  };

  void __PROC_QCL_IPC(handle val, XClientMessageEvent &XClient)
  {
    x11_message *TMsg;
    {
      u0 Temp = ((u0)XClient.data.l[1] << 32) | (u0)(XClient.data.l[0] & 0xffffffff);
      TMsg = (x11_message*)Temp;
    }
    x11_message XMsg = *TMsg;
    delete TMsg;



    switch (XMsg.Type)
    {
      case xmtMessage: goto _l_pro_Message;
      case xmtTask:    goto _l_pro_Task;
    }
    return;


    _l_pro_Message:
    switch (XMsg.Msg)
    {
      case controlMessages::cmPaint:
      {
        XEvent Cac;
        while (XCheckIfEvent(Native->dpy, &Cac, &__CheckDuplicate, (XPointer)&XMsg));
  

        XMsg.Ctrl->Handler_Message(XMsg.Msg);
        break;
      }
    
      default:
      {
        XMsg.Ctrl->Handler_Message(XMsg.Msg);
        break;
      }
    }
    return;


    _l_pro_Task:
    XMsg.Fun(XMsg.Data);
    return;
  }

  void __PROC_PROTOCOL(handle val, XClientMessageEvent &XClient)
  {
    if (XClient.data.l[0] == __CurrentApp->WDelete)
    {
      qcl::window *CWin = __FindWinFromWID(XClient.window);
      if (CWin == Nil)
        return;

      CWin->Handler_WindowClose();
    }

    #ifdef _QCL_DBG
    el
      cerr << "QCLDBG @qcl.x11 (Unknown X WM_PROTOCOLS Message) {.msg = " << XClient.message_type << "}" << endl;
    #endif
  }


  void Run(handle val) { while (true)
  {
    if (CurrentApp->WinList.empty())
      break;
    

    XEvent Ev;
    XNextEvent(Native->dpy, &Ev);



    if (Ev.type == ClientMessage)
    {
      if (Ev.xclient.message_type == __CurrentApp->qcl_ipc)
        __PROC_QCL_IPC(val, Ev.xclient);

      ef (Ev.xclient.message_type == __CurrentApp->WMProtocol)
        __PROC_PROTOCOL(val, Ev.xclient);


      #ifdef _QCL_DBG
      el
        cerr << "QCLDBG @qcl.x11 (Unknown X ClientMessage) {.msg = " << Ev.xclient.message_type << "}" << endl;
      #endif

      continue;
    }
    


    qcl::window *CWin = __FindWinFromWID(Ev.xany.window);
    if (CWin == Nil)
      continue;


    switch (Ev.type)
    {

      case Expose: // Paint
      {
        CWin->Handler_Paint();
        
        while (XCheckTypedWindowEvent(Native->dpy, Ev.xexpose.window, Expose, &Ev));
        break;
      }

      case ButtonPress: // Mouse Down
      {
        shiftStateSet LSta = XStateToQShift(Ev.xbutton.state);

        shiftStateSet LBut = 0;

        switch (Ev.xbutton.button)
        {
          case 1: LBut = shiftStates::ssLeft;   break;
          case 2: LBut = shiftStates::ssMiddle; break;
          case 3: LBut = shiftStates::ssRight;  break;

          case 4:
          case 5: {
            CWin->Handler_ScrollVert(poit_i32{Ev.xbutton.x, Ev.xbutton.y}, (Ev.xbutton.button == 4) ? (10):(-10), LSta);
            continue;
          };

          case 6:
          case 7: {
            CWin->Handler_ScrollHorz(poit_i32{Ev.xbutton.x, Ev.xbutton.y}, (Ev.xbutton.button == 6) ? (10):(-10), LSta);
            continue;
          };

          
          default: continue;
        }
        
        LSta &= ~LBut;

        CWin->Handler_MouseDown(poit_i32{Ev.xbutton.x, Ev.xbutton.y}, LBut, LSta);
        break;
      }

      case ButtonRelease: // Mouse Up
      {
        shiftStateSet LSta = XStateToQShift(Ev.xbutton.state);

        shiftStateSet LBut = 0;

        switch (Ev.xbutton.button)
        {
          case 1: LBut = shiftStates::ssLeft;     break;
          case 2: LBut = shiftStates::ssMiddle;   break;
          case 3: LBut = shiftStates::ssRight;    break;

          default: continue;
        }

        LSta &= ~LBut;

        CWin->Handler_MouseUp(poit_i32{Ev.xbutton.x, Ev.xbutton.y}, LBut, LSta);
        break;
      }

      case MotionNotify: // Mouse Move
      {
        shiftStateSet LSta = XStateToQShift(Ev.xmotion.state);

        CWin->Handler_MouseMove(poit_i32{Ev.xmotion.x, Ev.xmotion.y}, LSta);
        break;
      }  

      case KeyPress: // Key Down
      {
        shiftStateSet LSta = XStateToQShift(Ev.xkey.state);

        KeySym CKeySym = NoSymbol;
        Status CStatus;
        char Buf[8];


        int Len = Xutf8LookupString(((x11_handle*)CWin->OHID)->xic, &Ev.xkey, Buf, sizeof(Buf)-1, &CKeySym, &CStatus);

        if (Ev.xkey.state & ControlMask)
          printf("log: KDown CTRL + %s\n", XKeysymToString(CKeySym));
        
        ef (Len > 0) {
          Buf[Len] = '\0';
          
          CWin->Handler_KeyDown(Buf, 0, LSta);
          break;
        }

        ef (Len == 0) {
          Buf[0] = '\0';

          CWin->Handler_KeyDown(Buf, Ev.xkey.keycode, LSta);
          break;
        }

        break;
      }

      case KeyRelease: // Key Up
      {
        shiftStateSet LSta = XStateToQShift(Ev.xkey.state);

        KeySym CKeySym = NoSymbol;
        Status CStatus;
        char Buf[8];


        int Len = Xutf8LookupString(((x11_handle*)CWin->OHID)->xic, &Ev.xkey, Buf, sizeof(Buf)-1, &CKeySym, &CStatus);

        if (Ev.xkey.state & ControlMask)
          printf("log: KUp CTRL + %s\n", XKeysymToString(CKeySym));
        
        ef (Len > 0) {
          Buf[Len] = '\0';
          printf("log: KUp UTF-8 %s\n", Buf);
        }

        break;
      }

      case PropertyNotify: // Property
      {
        if (Atom NET_WM_STATE = XInternAtom(__CurrentApp->dpy, "_NET_WM_STATE", False); Ev.xproperty.atom == NET_WM_STATE)
        {
          Atom ActualType;
          int ActualFormat;
          unsigned long NItems, BytesAfter;
          u8 *Prop = Nil;

          Atom
            A_FullScr = XInternAtom(__CurrentApp->dpy, "_NET_WM_STATE_FULLSCREEN", False),
            A_MaxVert = XInternAtom(__CurrentApp->dpy, "_NET_WM_STATE_MAXIMIZED_VERT", False),
            A_MaxHorz = XInternAtom(__CurrentApp->dpy, "_NET_WM_STATE_MAXIMIZED_HORZ", False),
            A_Hidden  = XInternAtom(__CurrentApp->dpy, "_NET_WM_STATE_HIDDEN", False),
            A_Focused = XInternAtom(__CurrentApp->dpy, "_NET_WM_STATE_FOCUSED", False);

        
          if (XGetWindowProperty(
            __CurrentApp->dpy, ((x11_handle*)CWin->OHID)->window,
            NET_WM_STATE, 0, 1024, False, XA_ATOM,
            &ActualType, &ActualFormat, &NItems, &BytesAfter, &Prop
          ) == Success)
          {
            Atom *Atoms = (Atom*)Prop;
            bool
              Is_FullScr = false,
              Is_MaxVert = false,
              Is_MaxHorz = false,
              Is_Hidden  = false,
              Is_Focused = false;

            for (unsigned long i = 0; i < NItems; i++)
              if (Atoms[i] == A_FullScr) Is_FullScr = true;
              ef (Atoms[i] == A_MaxVert) Is_MaxVert = true;
              ef (Atoms[i] == A_MaxHorz) Is_MaxHorz = true;
              ef (Atoms[i] == A_Hidden)  Is_Hidden  = true;
              ef (Atoms[i] == A_Focused) Is_Focused = true;


            l_Change_WinState: {
              windowStates Temp = windowStates::wsNormal;

              if (Is_FullScr)
                Temp = windowStates::wsFullSrc;
              ef (Is_MaxVert && Is_MaxHorz)
                Temp = windowStates::wsMaximized;
              ef (Is_Hidden)
                Temp = windowStates::wsMinimized;
            

              if (Temp != CWin->WindowState)
                CWin->Handler_WindowStateChanged(Temp);
            }

            l_Change_Focus: {

              if (Is_Focused != bool(CWin->ControlState & controlStates::csFocus))
                CWin->Handler_StateChanged((CWin->ControlState & ~controlStates::csFocus) | (Is_Focused ? controlStates::csFocus : 0));
            }
            
            if (Prop)
              XFree(Prop);
          }
        }

        break;
      }

      case ConfigureNotify: // Configure: Resize (change bounds)
      {
        while (XCheckTypedWindowEvent(Native->dpy, Ev.xconfigure.window, ConfigureNotify, &Ev));

        ((x11_handle*)CWin->OHID)->posX = Ev.xconfigure.x;
        ((x11_handle*)CWin->OHID)->posY = Ev.xconfigure.y;

        if (Ev.xconfigure.width != CWin->Size.W || Ev.xconfigure.height != CWin->Size.H)
        {
          // Size
          CWin->Size = {Ev.xconfigure.width, Ev.xconfigure.height};

          ((x11_handle*)CWin->OHID)->width  = CWin->Size.W;
          ((x11_handle*)CWin->OHID)->height = CWin->Size.H;


          // Resize
          if (((x11_handle*)CWin->OHID)->Buf1 != 0)
            XFreePixmap(Native->dpy, ((x11_handle*)CWin->OHID)->Buf1);

          if (((x11_handle*)CWin->OHID)->Buf2 != 0)
            XFreePixmap(Native->dpy, ((x11_handle*)CWin->OHID)->Buf2);

          ((x11_handle*)CWin->OHID)->Buf1 = XCreatePixmap(
            Native->dpy,
            ((x11_handle*)CWin->OHID)->window,
            CWin->Size.W,
            CWin->Size.H,
            DefaultDepth(Native->dpy, ((x11_handle*)CWin->OHID)->screen)
          );

          ((x11_handle*)CWin->OHID)->Buf2 = XCreatePixmap(
            Native->dpy,
            ((x11_handle*)CWin->OHID)->window,
            CWin->Size.W,
            CWin->Size.H,
            DefaultDepth(Native->dpy, ((x11_handle*)CWin->OHID)->screen)
          );


          // Events
          CWin->Handler_Resize();
          CWin->Handler_Paint();
        }

        while (XCheckTypedWindowEvent(Native->dpy, Ev.xconfigure.window, Expose, &Ev));
        break;
      }

      
      #ifdef _QCL_DBG
      case MapNotify:
      case UnmapNotify:
      case NoExpose:
        break;

      default:
        cerr << "QCLDBG @qcl.x11 (Unknown X Message) {.msg = " << Ev.type << "}" << endl;
      #endif
    }
  }}

  #undef Native
}


namespace qcl::platform::opengl
{
  void New(handle *ret, u32 nWidth, u32 nHeight)
  {
    opengl_handle *h = (opengl_handle*)calloc(1, sizeof(opengl_handle));
    *ret = (handle)h;


    h->Width  = nWidth;
    h->Height = nHeight;

    // FBConfig
    int Attribs[] = {
      GLX_X_RENDERABLE, 1,
      GLX_DRAWABLE_TYPE, GLX_PBUFFER_BIT,
      GLX_RENDER_TYPE, GLX_RGBA_BIT,
      GLX_RED_SIZE, 8,
      GLX_GREEN_SIZE, 8,
      GLX_BLUE_SIZE, 8,
      GLX_ALPHA_SIZE, 8,
      None
    };
    int AttrC;

    h->FBConfigs = glXChooseFBConfig(__CurrentApp->dpy, DefaultScreen(__CurrentApp->dpy), Attribs, &AttrC);
    if (h->FBConfigs == Nil)
    {
      qcl_error("Can't open FBConfig");
      free(ret);
      *ret = (handle)Nil;
      return;
    }
    
    GLXFBConfig FBConfig = h->FBConfigs[0];


    // Visual Info
    XVisualInfo *VisInfo = glXGetVisualFromFBConfig(__CurrentApp->dpy, FBConfig);
    if (VisInfo == Nil)
    {
      qcl_error("Can't getted VisualInfo");
      free(ret);
      *ret = (handle)Nil;
      return;
    }



    // GLX
    h->Context = glXCreateNewContext(__CurrentApp->dpy, FBConfig, GLX_RGBA_TYPE, Nil, true);
    if (h->Context == Nil)
    {
      qcl_error("Can't created GLX context");
      free(ret);
      *ret = (handle)Nil;
      return;
    }



    // PBuffer
    int PBufferAttribs[] = {
      GLX_PBUFFER_WIDTH, (int)h->Width,
      GLX_PBUFFER_HEIGHT, (int)h->Height,
      None
    };

    h->PBuffer = glXCreatePbuffer(__CurrentApp->dpy, FBConfig, PBufferAttribs);
    if (h->PBuffer == 0)
    {
      qcl_error("Can't create PBuffer");
      free(ret);
      *ret = (handle)Nil;
      return;
    }



    // Connect
    if (!glXMakeContextCurrent(__CurrentApp->dpy, h->PBuffer, h->PBuffer, h->Context))
    {
      qcl_error("Can't connected PBuffer");
      free(ret);
      *ret = (handle)Nil;
      return;
    }



    // GLew
    GLenum err = glewInit();
    if (GLEW_OK != err)
    {
      qcl_error(("GLEW init failed: " + string((const char*)glewGetErrorString(err))).c_str());
      free(ret);
      *ret = (handle)Nil;
      return;
    }


    // FBO + Texture
    glGenFramebuffers(1, &h->FBO);
    glBindFramebuffer(GL_FRAMEBUFFER, h->FBO);

    glGenTextures(1, &h->Tex);
    glBindTexture(GL_TEXTURE_2D, h->Tex);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE);
    glTexImage2D(GL_TEXTURE_2D, 0, GL_RGBA8, h->Width, h->Height, 0, GL_RGBA, GL_UNSIGNED_BYTE, nullptr);

    glFramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, GL_TEXTURE_2D, h->Tex, 0);

    if (glCheckFramebufferStatus(GL_FRAMEBUFFER) != GL_FRAMEBUFFER_COMPLETE)
    {
      qcl_error("Can't created FBO");
      free(ret);
      *ret = (handle)Nil;
      return;
    }

    glViewport(0, 0, h->Width, h->Height);
    XFree(VisInfo);
  }

  #define Native ((opengl_handle*)val)

  void Dis(handle  val)
  {
    glXMakeContextCurrent(__CurrentApp->dpy, 0, 0, Nil);
    glXDestroyPbuffer(__CurrentApp->dpy, Native->PBuffer);
    glXDestroyContext(__CurrentApp->dpy, Native->Context);
    XFree(Native->FBConfigs);

    free(Native);
  }


  void Set_Current(handle  val)
  {
    if (glXGetCurrentContext() == Native->Context)
      return;

    if (!glXMakeContextCurrent(__CurrentApp->dpy, Native->PBuffer, Native->PBuffer, Native->Context))
    {
      qcl_error("Can't connected PBuffer");
      return;
    }
  }

  void Set_Size(handle  val, u32 nWidth, u32 nHeight)
  {
    glDeleteFramebuffers(1, &Native->FBO);
    glDeleteTextures(1, &Native->Tex);


    // New OpenGL
    Native->Width  = max<u32>(10, nWidth);
    Native->Height = max<u32>(10, nHeight);

    glGenTextures(1, &Native->Tex);
    glBindTexture(GL_TEXTURE_2D, Native->Tex);
    glTexImage2D(GL_TEXTURE_2D, 0, GL_RGBA, Native->Width, Native->Height, 0, GL_BGRA, GL_UNSIGNED_BYTE, Nil);

    glGenFramebuffers(1, &Native->FBO);
    glBindFramebuffer(GL_FRAMEBUFFER, Native->FBO);
    glFramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, GL_TEXTURE_2D, Native->Tex, 0);

    if (glCheckFramebufferStatus(GL_FRAMEBUFFER) != GL_FRAMEBUFFER_COMPLETE)
    {
      qcl_error("Can't created FBO");
      return;
    }

    glViewport(0, 0, Native->Width, Native->Height);
  }


  void ReadBuffer(handle  val, u32 *PixBuf)
  {
    glReadPixels(
      0,0,
      Native->Width, Native->Height,
      GL_BGRA,
      GL_UNSIGNED_BYTE,
      PixBuf
    );
  }

  void Flush(handle  val)
  {
    glFlush();
  }

  #undef Native
}


namespace qcl::platform::dialog // use zenity
{

  char* FileOpen(const char *Title, const char *Filters[])  // with malloc
  {
    string Cac = "zenity --file-selection";

    if (Title != Nil)
      Cac += " --title=\""+string(Title)+"\"";

    
    if (Filters != Nil)
    {
      int i = 0;
      while (Filters[i] != Nil)
      {
        Cac += " --file-filter=\""+string(Filters[i])+"\"";
        i++;
      }
    }


    FILE *fp = popen(Cac.c_str(), "r");
    if (fp == Nil)
      return Nil;


    string Ret;

    char c;
    while (fread(&c, 1, 1, fp) == 1)
      Ret += c;

    int ret = pclose(fp);
    if (ret != 0 || Ret.empty())
      return Nil;


    // sonundaki \n karakterini kaldır
    if (!Ret.empty() && Ret.back() == '\n')
      Ret.pop_back();


    char *Buf = (char*)malloc(Ret.size()+1);

    memcpy(Buf, Ret.c_str(), Ret.size()+1);

    return Buf;
  }

  char* FileSave(const char *Title, const char *Filters[])  // with malloc
  {
    string Cac = "zenity --file-selection --save";

    if (Title != Nil)
      Cac += " --title=\""+string(Title)+"\"";

    
    if (Filters != Nil)
    {
      int i = 0;
      while (Filters[i] != Nil)
      {
        Cac += " --file-filter=\""+string(Filters[i])+"\"";
        i++;
      }
    }


    FILE *fp = popen(Cac.c_str(), "r");
    if (fp == Nil)
      return Nil;


    string Ret;

    char c;
    while (fread(&c, 1, 1, fp) == 1)
      Ret += c;

    int ret = pclose(fp);
    if (ret != 0 || Ret.empty())
      return Nil;


    // sonundaki \n karakterini kaldır
    if (!Ret.empty() && Ret.back() == '\n')
      Ret.pop_back();


    char *Buf = (char*)malloc(Ret.size()+1);

    memcpy(Buf, Ret.c_str(), Ret.size()+1);

    return Buf;
  }

  void Message(const char *Text)
  {
    string Cac = "zenity --info --text=\""+string(Text)+"\"";


    FILE *fp = popen(Cac.c_str(), "r");
    if (fp == Nil)
      return;

    pclose(fp);
  }

}

#undef __CurrentApp



[[gnu::weak]] int main()
{
  if (qcl::platform::qcl_entry1)
  {
    ohid App_H{};
    qcl::platform::application::New(&App_H);

    ohid Win_H{};
    qcl::platform::window::New(&Win_H);
    
    return qcl::platform::qcl_entry1(App_H, Win_H);
  }

  ef (qcl::platform::qcl_entry2)
  {
    ohid App_H{};
    qcl::platform::application::New(&App_H);
    
    return qcl::platform::qcl_entry2(App_H);
  }

  ef (qcl::platform::qcl_entry3)
  {
    return qcl::platform::qcl_entry3();
  }

  el {
    #ifdef _DBG
    cerr << "Not found a qcl entry" << endl;
    #endif

    return -1;
  }
}

