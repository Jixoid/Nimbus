/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/



#pragma once

#include <expected>
#include <string>
#include <string_view>
#include <vector>

#include "DS.hh"
#include "include/core/SkColor.h"
#include "include/core/SkFont.h"
#include "include/core/SkFontMetrics.h"
#include "include/core/SkFontTypes.h"
#include "include/core/SkMaskFilter.h"
#include "include/core/SkRefCnt.h"
#include "include/core/SkSurface.h"
#include "include/core/SkPaint.h"
#include "include/core/SkRRect.h"
#include "include/core/SkTypeface.h"
#include "include/effects/SkBlurMaskFilter.h"
#include "qcl/Object.hh"
#include "qcl/Scrolling.hh"
#include "qcl/Types.hh"
#include "qcl/Visual.hh"
#include "qcl/Widget.hh"
#include "qcl/Window.hh"
#include "qcl/DynamicLoad.hh"
#include "qcl/Platform.hh"
#include "qcl/Popup.hh"
#include "qstd/Monet.hh"

using namespace std;
using namespace qcl;



namespace qstd
{
  fun objs() -> dyn::objs;


  struct qcl_object form: qcl::window
  {
    private:
      dcolor m_color{this, monetRole::SurfaceContainer};
      
    public:
      inline fun color() -> dcolor& { return m_color; }
      inline fun propColor(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isString()) return std::unexpected("was expected string");

        m_color.update(Prop.w_string());

        return true;
      }
      qcl_prop(TYPE color, NAME color, PROP propColor);
    
    public:
      fun draw(renderContext&) -> void override;

      fun doReset(renderContext&) -> void override;
      
    public:
      fun getProp(string_view) -> expected<any, bool> override;
      fun setProp(string_view, any) -> expected<void, bool> override;
      fun loadProp(string_view, const ds::value&, function<expected<qev_seed,bool>(string_view)>) -> expected<bool, string> override;
  };


  struct qcl_object layout: qcl::scrolling
  {
    private:
      dcolor m_color{this, monetRole::SurfaceContainerLow};
      f32 m_borderRadius{12};

    public:
      inline fun color() -> dcolor& { return m_color; }
      inline fun propColor(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isString()) return std::unexpected("was expected string");

        m_color.update(Prop.w_string());

        return true;
      }
      qcl_prop(TYPE color, NAME color, PROP propColor);


      inline fun borderRadius() const noexcept -> f32 { return m_borderRadius; }
      inline fun setBorderRadius(f32 val) -> void { m_borderRadius = val; update(visDirtyDraw); }
      inline fun propBorderRadius(const ds::value& Prop) -> expected<bool, string> {
        
        if (Prop.isFloat())
          setBorderRadius(Prop.w_float());
        ef (Prop.isInt())
          setBorderRadius(Prop.w_int());
        else
          return std::unexpected("was expected float or int");

        return true;
      }
      qcl_prop(TYPE f32, NAME borderRadius, READ borderRadius, WRITE setBorderRadius, PROP propBorderRadius);


    public:
      fun draw(renderContext&) -> void override;

      fun draw_scrollVert(renderContext&) -> void override;
      fun draw_scrollHorz(renderContext&) -> void override;

      fun doReset(renderContext&) -> void override;

    public:
      fun getProp(string_view) -> expected<any, bool> override;
      fun setProp(string_view, any) -> expected<void, bool> override;
      fun loadProp(string_view, const ds::value&, function<expected<qev_seed,bool>(string_view)>) -> expected<bool, string> override;
  };

  struct layout_vert: qstd::layout
  {
    fun doTiling(renderContext&) -> void override;
  };

  struct layout_horz: qstd::layout
  {
    fun doTiling(renderContext&) -> void override;
  };

  struct layout_flow: qstd::layout
  {
    fun doTiling(renderContext&) -> void override;
  };



  enum dialog_titleButtonType
  {
    dtbtNone  = 0,
    dtbtClose = 1,
    dtbtMax   = 2,
    dtbtMin   = 3,
  };

  struct dialog_titleButton: qcl::widget
  {
    public:
      dialog_titleButton()
      {
        setSize({20,20});
      }

    private:
      dialog_titleButtonType m_buttonType{};

    public:
      inline fun buttonType() -> dialog_titleButtonType { return m_buttonType; }
      inline fun setButtonType(dialog_titleButtonType val) { m_buttonType = val; update(); }


    public:
      inline fun draw(renderContext& Context) -> void override { switch (m_buttonType) {
        case dtbtClose: {
          auto Canvas = Context.canvas();
          SkPaint Paint(Context.paint());
          Paint.setColor4f(Monet.get(monetRole::OnSurfaceContainerLow));

          f32 X = (f32)width();
          Canvas->drawLine({X/5*1, X/5*1}, {X/5*4, X/5*4}, Paint);
          Canvas->drawLine({X/5*4, X/5*1}, {X/5*1, X/5*4}, Paint);
          break;
        }

        case dtbtMin: {
          auto Canvas = Context.canvas();
          SkPaint Paint(Context.paint());
          Paint.setColor4f(Monet.get(monetRole::OnSurfaceContainerLow));

          f32 X = (f32)width();
          Canvas->drawLine({X/6*1, X/9*3}, {X/6*3, X/9*6}, Paint);
          Canvas->drawLine({X/6*3, X/9*6}, {X/6*5, X/9*3}, Paint);
          break;
        }

        case dtbtMax: {
          auto Canvas = Context.canvas();
          SkPaint Paint(Context.paint());
          Paint.setColor4f(Monet.get(monetRole::OnSurfaceContainerLow));

          f32 X = (f32)width();
          Canvas->drawLine({X/6*1, X/9*6}, {X/6*3, X/9*3}, Paint);
          Canvas->drawLine({X/6*3, X/9*3}, {X/6*5, X/9*6}, Paint);
          break;
        }

        default:
      }}
  };

  struct dialog_titleBar: qcl::view
  {
    public:
      inline dialog_titleBar(): m_btnClose(new dialog_titleButton())
      {
        setButtons();
        setSize({0, 30});
      }

    private:
      string m_text{"dialog"};

    private:
      fun setButtons() -> void;

      dialog_titleButton *m_btnClose{};


    public:
      inline fun text() const -> string { return m_text; }
      inline fun setText(string_view val) -> void { m_text = val; update(); }

    public:
      inline fun draw(renderContext& Context) -> void override
      {
        auto Surface = Context.canvas();
        auto Paint = SkPaint(Context.paint());
        auto Font = SkFont(Context.fontType());
        Font.setSize(Context.sp(16));
        SkFontMetrics Metrics;
        Font.getMetrics(&Metrics);

        SkVector radii[4] = {{16,16},{16,16}, {0,0},{0,0}};
        SkRRect RRect;
        RRect.setRectRadii(SkRect::MakeLTRB(0,0, width(), height()), radii);

        Paint.setColor4f(Monet.get(monetRole::SurfaceContainerLow));
        Surface->drawRRect(RRect, Paint);

        SkRect TSize;
        Font.measureText(m_text.c_str(), m_text.size(), SkTextEncoding::kUTF8, &TSize);


        poit<i32> Pos;
        Pos.Y = (height() / 2.0f) - ((Metrics.fAscent + Metrics.fDescent) / 2.0f);
        Pos.X = Pos.Y/2;
        Paint.setColor4f(Monet.get(monetRole::OnSurfaceContainerLow));
        Paint.setStyle(SkPaint::kFill_Style);
        Surface->drawString(m_text.c_str(), Pos.X, Pos.Y, Font, Paint);
      }

      inline fun doMouseDown(poit<f32> Pos, shiftStateSet Button, shiftStateSet State) -> void override
      {
        if (auto Win = dynamic_cast<window*>(getRoot()); Win && Button == shiftStates::ssLeft)
        {
          platform::API->Window.startResize(Win->ohid(), {0,0}, platform::resizeOp::wrlMove);
        }
      }
  };


  struct dialog: qcl::view
  {
    public:
      inline dialog(): m_titleBar(new dialog_titleBar())
      {
        setTitleBar();
        setSize({400, 240});
      }

    private:
      dialog_titleBar *m_titleBar{};
      fun setTitleBar() -> void;


    public:
      inline fun titleBar() const noexcept -> dialog_titleBar* { return m_titleBar; }


    public:
      fun showModal() -> void;

      fun draw(renderContext&) -> void override;

  };


  enum text_align: u8
  {
    taLeft = 1,
    taTop  = 1,

    taCenter = 2,

    taRight  = 3,
    taBottom = 3,
  };

  struct qcl_object text: qcl::widget
  {
    public:
      inline text() { setAutoSize(true); }


    private:
      dcolor m_color{this, monetRole::OnSurfaceContainer};
      sk_sp<SkTypeface> m_font;
      
      string m_text{"text"};

      f32 m_fontSize{14};
      f32 m_fontGrad{0};
      f32 m_fontWght{0};
      f32 m_fontWdth{0};

      text_align m_alignVert{taCenter};
      text_align m_alignHorz{taCenter};

    public:
      inline fun color() -> dcolor& { return m_color; }
      inline fun propColor(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isString()) return std::unexpected("was expected string");

        m_color.update(Prop.w_string());

        return true;
      }
      qcl_prop(TYPE color, NAME color, PROP propColor);

      inline fun getText() const -> string { return m_text; }
      inline fun setText(string val) -> void { m_text = val; update(visDirtyAutoSize | visDirtyDraw); }
      inline fun propText(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isString()) return std::unexpected("was expected string");

        setText(Prop.w_string());
        return true;
      }
      qcl_prop(TYPE string, NAME text, READ getText, WRITE setText, PROP propText);
      

      inline fun fontSize() const noexcept -> f32 { return m_fontSize; }
      inline fun setFontSize(f32 val) -> void { m_fontSize = val; update(visDirtyAutoSize | visDirtyDraw); }
      inline fun propFontSize(const ds::value& Prop) -> expected<bool, string> {
        if (Prop.isInt()) setFontSize(Prop.w_int());
        ef (Prop.isFloat()) setFontSize(Prop.w_float());
        else
          return std::unexpected("was expected int or float");

        return true;
      }
      qcl_prop(TYPE f32, NAME fontSize, READ fontSize, WRITE setFontSize, PROP propFontSize);


      inline fun fontGrad() const noexcept -> f32 { return m_fontGrad; }
      inline fun setFontGrad(f32 val) -> void { m_fontGrad = val; update(visDirtyReset | visDirtyAutoSize | visDirtyDraw); }
      inline fun propFontGrad(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isFloat())
          return std::unexpected("was expected float");
        
        setFontGrad(Prop.w_float());
        return true;
      }
      qcl_prop(TYPE f32, NAME fontGrad, READ fontGrad, WRITE setFontGrad, PROP propFontGrad);


      inline fun fontWght() const noexcept -> f32 { return m_fontWght; }
      inline fun setFontWght(f32 val) -> void { m_fontWght = val; update(visDirtyAutoSize | visDirtyDraw); }
      inline fun propFontWght(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isFloat())
          return std::unexpected("was expected float");
      
        setFontWght(Prop.w_float());
        return true;
      }
      qcl_prop(TYPE f32, NAME fontWght, READ fontWght, WRITE setFontWght, PROP propFontWght);


      inline fun fontWdth() const noexcept -> f32 { return m_fontWdth; }
      inline fun setFontWdth(f32 val) -> void { m_fontWdth = val; update(visDirtyAutoSize | visDirtyDraw); }
      inline fun propFontWdth(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isFloat())
          return std::unexpected("was expected float");
      
        setFontWdth(Prop.w_float());
        return true;
      }
      qcl_prop(TYPE f32, NAME fontWdth, READ fontWdth, WRITE setFontWdth, PROP propFontWdth);


      inline fun alignVert() const noexcept -> text_align { return m_alignVert; }
      inline fun setAlignVert(text_align val) -> void { m_alignVert = val; update(visDirtyAutoSize); }
      inline fun propAlignVert(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isString()) return std::unexpected("was expected string");

        auto Side = Prop.w_string();

        if (Side == "<-")
          setAlignVert(text_align::taLeft);
        ef (Side == "->")
          setAlignVert(text_align::taRight);
        ef (Side == "><")
          setAlignVert(text_align::taCenter);
        else
          return std::unexpected("invalid side (<-, ->, ><)");

        return true;
      }
      qcl_prop(TYPE text_align, NAME alignVert, READ alignVert, WRITE setAlignVert, PROP propAlignVert);

      
      inline fun alignHorz() const noexcept -> text_align { return m_alignHorz; }
      inline fun setAlignHorz(text_align val) -> void { m_alignHorz = val; update(visDirtyAutoSize); }
      inline fun propAlignHorz(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isString()) return std::unexpected("was expected string");

        auto Side = Prop.w_string();

        if (Side == "<-")
          setAlignHorz(text_align::taLeft);
        ef (Side == "->")
          setAlignHorz(text_align::taRight);
        ef (Side == "><")
          setAlignHorz(text_align::taCenter);
        else
          return std::unexpected("invalid side (<-, ->, ><)");

        return true;
      }
      qcl_prop(TYPE text_align, NAME alignHorz, READ alignHorz, WRITE setAlignHorz, PROP propAlignHorz);


    public:
      fun draw(renderContext&) -> void override;

      fun calcAutoSize(renderContext&) -> void override;

      fun doReset(renderContext&) -> void override;

    public:
      fun getProp(string_view) -> expected<any, bool> override;
      fun setProp(string_view, any) -> expected<void, bool> override;
      fun loadProp(string_view, const ds::value&, function<expected<qev_seed,bool>(string_view)>) -> expected<bool, string> override;
  };

  struct qcl_object icon: qcl::widget
  {
    private:
      sk_sp<SkSurface> m_icon{};

      dcolor m_theme{this, monetRole::OnSurface};

      string m_path = "";

      bool m_reRender{false};


    public:
      inline fun theme() -> dcolor& { return m_theme; }
      inline fun propTheme(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isString()) return std::unexpected("was expected string");

        m_theme.update(Prop.w_string());

        return true;
      }
      qcl_prop(TYPE color, NAME theme, PROP propTheme);


      inline fun path() const -> string { return m_path; }
      inline fun setPath(string val) -> void { m_path = val; m_reRender = true; update(visDirtyDraw); }
      inline fun propPath(const ds::value& Prop) -> std::expected<bool, std::string> {
        if (!Prop.isString()) return std::unexpected("was expected string");

        setPath(Prop.w_string());
        return true;
      }
      qcl_prop(TYPE string, NAME path, READ path, WRITE setPath, PROP propPath);

    public:
      fun draw(renderContext&) -> void override;

      fun doPaint_prepare(renderContext&) -> void override;

    public:
      fun getProp(string_view) -> expected<any, bool> override;
      fun setProp(string_view, any) -> expected<void, bool> override;
      fun loadProp(string_view, const ds::value&, function<expected<qev_seed,bool>(string_view)>) -> expected<bool, string> override;
  };

  struct qcl_object edit: qcl::widget
  {
    public:
      inline edit() { setSize({160, 40}); }

    private:
      string m_text{}, m_title{};
      u16 m_borderRadius{6};

      sk_sp<SkTypeface> m_font{};
      float m_scrollX{};

      i0 m_pos{}, m_pos2{-1};

    public:
      inline fun text() const -> string { return m_text; }
      inline fun setText(string val) -> void { m_text = val; update(visDirtyDraw); }
      inline fun propText(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isString()) return std::unexpected("was expected string");

        setText(Prop.w_string());
        return true;
      }
      qcl_prop(TYPE string, NAME text, READ text, WRITE setText, PROP propText);
      

      inline fun title() const -> string { return m_title; }
      inline fun setTitle(string val) -> void { m_title = val; update(visDirtyDraw); }
      inline fun propTitle(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isString()) return std::unexpected("was expected string");

        setTitle(Prop.w_string());
        return true;
      }
      qcl_prop(TYPE string, NAME title, READ title, WRITE setTitle, PROP propTitle);
      

      inline fun borderRadius() const noexcept -> u16 { return m_borderRadius; }
      inline fun setBorderRadius(u16 val) -> void { m_borderRadius = val; update(visDirtyDraw); }
      inline fun propBorderRadius(const ds::value& Prop) -> expected<bool, string> {
        
        if (Prop.isFloat())
          setBorderRadius(Prop.w_float());
        ef (Prop.isInt())
          setBorderRadius(Prop.w_int());
        else
          return std::unexpected("was expected float or int");

        return true;
      }
      qcl_prop(TYPE u16, NAME borderRadius, READ borderRadius, WRITE setBorderRadius, PROP propBorderRadius);

    public:
      fun draw(renderContext&) -> void override;

      fun handlerStateChanged(qcl::visual::stateFlagSet State) -> void override;

      fun doMouseDown(poit<f32> Pos, shiftStateSet Button, shiftStateSet State) -> void override;
      fun doMouseMove(poit<f32> Pos, shiftStateSet State) -> void override;
      fun doKeyDown(u64 Key, u32 KeyCode, qcl::shiftStateSet State) -> void override;

    public:
      fun getProp(string_view) -> expected<any, bool> override;
      fun setProp(string_view, any) -> expected<void, bool> override;
      fun loadProp(string_view, const ds::value&, function<expected<qev_seed,bool>(string_view)>) -> expected<bool, string> override;
  };

  struct qcl_object radio: qcl::widget
  {
    public:
      inline radio() { setAutoSize(true); }


    private:
      string m_text{"radio"};

      bool m_checked{false};
  
    public:
      inline fun text() const -> string { return m_text; }
      inline fun setText(string val) -> void { m_text = val; update(visDirtyDraw); }
      inline fun propText(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isString()) return std::unexpected("was expected string");

        setText(Prop.w_string());
        return true;
      }
      qcl_prop(TYPE string, NAME text, READ text, WRITE setText, PROP propText);

      
      inline fun checked() const noexcept -> bool { return m_checked; }
      inline fun setChecked(bool val) -> void { m_checked = val; update(visDirtyDraw); }
      inline fun propChecked(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isBool())
          return std::unexpected("was expected bool");
      
        setChecked(Prop.w_bool());
        return true;
      }
      qcl_prop(TYPE bool, NAME checked, READ checked, WRITE setChecked, PROP propChecked);


    public:
      qcl_func qev<void, bool /* Status */> onChanged;
    
    public:
      fun draw(renderContext&) -> void override;

      fun handlerStateChanged(qcl::visual::stateFlagSet State) -> void override;

      fun doClick() -> void override;
      virtual fun doChanged(bool Status) -> void;

      fun calcAutoSize(renderContext&) -> void override;

    public:
      fun getProp(string_view) -> expected<any, bool> override;
      fun setProp(string_view, any) -> expected<void, bool> override;
      fun loadProp(string_view, const ds::value&, function<expected<qev_seed,bool>(string_view)>) -> expected<bool, string> override;
  };

  struct qcl_object check: qcl::widget
  {
    public:
      inline check() { setAutoSize(true); }


    private:
      string m_text{"check"};

      bool m_checked{false};
  
    public:
      inline fun text() const -> string { return m_text; }
      inline fun setText(string val) -> void { m_text = val; update(visDirtyDraw); }
      inline fun propText(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isString()) return std::unexpected("was expected string");

        setText(Prop.w_string());
        return true;
      }
      qcl_prop(TYPE string, NAME text, READ text, WRITE setText, PROP propText);

      
      inline fun checked() const noexcept -> bool { return m_checked; }
      inline fun setChecked(bool val) -> void { m_checked = val; update(visDirtyDraw); }
      inline fun propChecked(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isBool())
          return std::unexpected("was expected bool");
      
        setChecked(Prop.w_bool());
        return true;
      }
      qcl_prop(TYPE bool, NAME checked, READ checked, WRITE setChecked, PROP propChecked);


    public:
      qcl_func qev<void, bool /* Status */> onChanged;


    public:
      fun draw(renderContext&) -> void override;

      fun handlerStateChanged(qcl::visual::stateFlagSet State) -> void override;

      fun doClick() -> void override;
      virtual fun doChanged(bool Status) -> void;

      fun calcAutoSize(renderContext&) -> void override;

    public:
      fun getProp(string_view) -> expected<any, bool> override;
      fun setProp(string_view, any) -> expected<void, bool> override;
      fun loadProp(string_view, const ds::value&, function<expected<qev_seed,bool>(string_view)>) -> expected<bool, string> override;
  };

  struct qcl_object toggle: qcl::widget
  {
    public:
      inline toggle() { setAutoSize(true); }


    private:
      string m_text{"toggle"};

      bool m_checked{false};
  
    public:
      inline fun text() const -> string { return m_text; }
      inline fun setText(string val) -> void { m_text = val; update(visDirtyDraw); }
      inline fun propText(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isString()) return std::unexpected("was expected string");

        setText(Prop.w_string());
        return true;
      }
      qcl_prop(TYPE string, NAME text, READ text, WRITE setText, PROP propText);

      
      inline fun checked() const noexcept -> bool { return m_checked; }
      inline fun setChecked(bool val) -> void { m_checked = val; update(visDirtyDraw); }
      inline fun propChecked(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isBool())
          return std::unexpected("was expected bool");
      
        setChecked(Prop.w_bool());
        return true;
      }
      qcl_prop(TYPE bool, NAME checked, READ checked, WRITE setChecked, PROP propChecked);


    public:
      qcl_func qev<void, bool /* Status */> onChanged;


    public:
      fun draw(renderContext&) -> void override;

      fun handlerStateChanged(qcl::visual::stateFlagSet State) -> void override;

      fun doClick() -> void override;
      virtual fun doChanged(bool Status) -> void;

      fun calcAutoSize(renderContext&) -> void override;

    public:
      fun getProp(string_view) -> expected<any, bool> override;
      fun setProp(string_view, any) -> expected<void, bool> override;
      fun loadProp(string_view, const ds::value&, function<expected<qev_seed,bool>(string_view)>) -> expected<bool, string> override;
  };

  struct qcl_object progbar: qcl::widget
  {
    public:
      inline progbar() { setSize({140, 24}); }


    private:
      string m_text{"progbar"};
      sk_sp<SkTypeface> m_font;

      u32 m_max{100}, m_value{30};


    public:
      inline fun text() const -> string { return m_text; }
      inline fun setText(string val) -> void { m_text = val; update(visDirtyDraw); }
      inline fun propText(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isString()) return std::unexpected("was expected string");

        setText(Prop.w_string());
        return true;
      }
      qcl_prop(TYPE string, NAME text, READ text, WRITE setText, PROP propText);

      
      inline fun max() const noexcept -> u32 { return m_max; }
      inline fun setMax(u32 val) -> void { m_max = val; update(visDirtyDraw); }
      inline fun propMax(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isInt())
          return std::unexpected("was expected int");
      
        setMax(Prop.w_int());
        return true;
      }
      qcl_prop(TYPE u32, NAME max, READ max, WRITE setMax, PROP propMax);

      
      inline fun value() const noexcept -> u32 { return m_value; }
      inline fun setValue(u32 val) -> void { m_value = val; update(visDirtyDraw); }
      inline fun propValue(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isInt())
          return std::unexpected("was expected int");
      
        setValue(Prop.w_int());
        return true;
      }
      qcl_prop(TYPE u32, NAME value, READ value, WRITE setValue, PROP propValue);
      

    public:
      fun draw(renderContext&) -> void override;

    public:
      fun getProp(string_view) -> expected<any, bool> override;
      fun setProp(string_view, any) -> expected<void, bool> override;
      fun loadProp(string_view, const ds::value&, function<expected<qev_seed,bool>(string_view)>) -> expected<bool, string> override;
  };

  struct qcl_object slider: qcl::widget
  {
    public:
      inline slider() { setSize({140, 28}); }
      

    private:
      u32 m_max{100}, m_value{30};


    public:
      inline fun max() const noexcept -> u32 { return m_max; }
      inline fun setMax(u32 val) -> void { if (m_max == val) return; m_max = val; update(visDirtyDraw); }
      inline fun propMax(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isInt())
          return std::unexpected("was expected int");
      
        setMax(Prop.w_int());
        return true;
      }
      qcl_prop(TYPE u32, NAME max, READ max, WRITE setMax, PROP propMax);

      
      inline fun value() const noexcept -> u32 { return m_value; }
      inline fun setValue(u32 val) -> void { if (m_value == val) return; m_value = val; update(visDirtyDraw); }
      inline fun propValue(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isInt())
          return std::unexpected("was expected int");
      
        setValue(Prop.w_int());
        return true;
      }
      qcl_prop(TYPE u32, NAME value, READ value, WRITE setValue, PROP propValue);


    public:
      qcl_func qev<void, u32 /* Value */> onChanged;


    public:
      fun draw(renderContext&) -> void override;

      fun handlerStateChanged(qcl::visual::stateFlagSet State) -> void override;
      fun doMouseDown(poit<f32> Pos, shiftStateSet Button, shiftStateSet State) -> void override;
      fun doMouseMove(poit<f32> Pos, shiftStateSet State) -> void override;
      virtual fun doChanged(u32 Value) -> void;

    public:
      fun getProp(string_view) -> expected<any, bool> override;
      fun setProp(string_view, any) -> expected<void, bool> override;
      fun loadProp(string_view, const ds::value&, function<expected<qev_seed,bool>(string_view)>) -> expected<bool, string> override;
  };

  struct qcl_object tabs: qcl::widget
  {
    public:
      inline tabs() { setAutoSize(true); }

    protected:
      vector<string> m_tabs{};
      vector<qcl::size<f32>> m_optTabs{};

      i32 m_tabID{-1};
      i32 m_hTabID{-1};

    public:
      inline fun getTabs() const -> vector<string> { return m_tabs; }
      inline fun setTabs(vector<string> val) -> void { m_tabs = val; update(visDirtyAutoSize | visDirtyDraw); }
      inline fun propTabs(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isArray()) return std::unexpected("was expected array");

        vector<string> Nat;

        for (int i{}; i < Prop.size(); i++) {
          if (auto X = Prop[i]; X.isString())
            Nat.push_back(X.w_string());
          else
            return std::unexpected("was expected string, in /["+to_string(i+1)+"]");
        }

        setTabs(Nat);
        return true;
      }
      qcl_prop(TYPE vector<string>, NAME tabs, READ getTabs, WRITE setTabs, PROP propTabs);


      inline fun tabID() const noexcept -> i32 { return m_tabID; }
      inline fun setTabID(i32 val) -> void { m_tabID = val; update(visDirtyDraw); }
      inline fun propTabID(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isInt()) return std::unexpected("was expected int");

        setTabID(Prop.w_int());

        return true;
      }
      qcl_prop(TYPE i32, NAME tabID, READ tabID, WRITE setTabID, PROP propTabID);
      

    public:
      qcl_func qev<void, i32 /* TabID */> onChanged;

    public:
      fun draw(renderContext&) -> void override;

      fun handlerStateChanged(qcl::visual::stateFlagSet State) -> void override;
      
      fun doMouseMove(poit<f32> Pos, shiftStateSet State) -> void override;
      fun doClickEx(poit<f32> Pos) -> void override;
      virtual fun doChanged(i32 nTabID) -> void;
      
      fun calcAutoSize(renderContext&) -> void override;
      fun recalcTabsSize(renderContext&) -> void;
      
    public:
      fun getProp(string_view) -> expected<any, bool> override;
      fun setProp(string_view, any) -> expected<void, bool> override;
      fun loadProp(string_view, const ds::value&, function<expected<qev_seed,bool>(string_view)>) -> expected<bool, string> override;
  };

  struct qcl_object choice: qstd::tabs
  {
    private:
      u32 m_threshold{0};

    public:
      inline fun threshold() const noexcept -> u32 { return m_threshold; }
      inline fun setThreshold(u32 val) -> void { m_threshold = val; update(visDirtyDraw); }
      inline fun propThreshold(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isInt())
          return std::unexpected("was expected int");
      
        setThreshold(Prop.w_int());
        return true;
      }
      qcl_prop(TYPE u32, NAME threshold, READ threshold, WRITE setThreshold, PROP propThreshold);

    public:
      fun draw(renderContext&) -> void override;

      fun doMouseMove(poit<f32> Pos, shiftStateSet State) -> void override;
      fun doClickEx(poit<f32> Pos) -> void override;

      fun calcAutoSize(renderContext&) -> void override;

    public:
      fun getProp(string_view) -> expected<any, bool> override;
      fun setProp(string_view, any) -> expected<void, bool> override;
      fun loadProp(string_view, const ds::value&, function<expected<qev_seed,bool>(string_view)>) -> expected<bool, string> override;
  };


  enum grid_col_type 
  {
    gctText     = 1,
    gctCheckbox = 2,
    gctButton   = 3,
  };

  struct grid_column
  {
    string Title;
    string FieldName;
    i32 Width = 100;
    
    grid_col_type Type = grid_col_type::gctText;
  };

  struct qcl_object grid: qcl::widget
  {
    private:
      vector<grid_column> m_cols;
      vector<vector<string>> m_rows;

      u32 m_rowHeight{24};
      u32 m_headerHeight{30};

      dcolor m_lineColor{this, "monet(SurfaceVariant)"};

    public:
      inline fun cols() const -> const vector<grid_column>& { return m_cols; }
      inline fun setCols(const vector<grid_column>& val) -> void { m_cols = val; update(visDirtyDraw); }

      inline fun rows() const -> const vector<vector<string>>& { return m_rows; }
      inline fun setRows(const vector<vector<string>>& val) -> void { m_rows = val; update(visDirtyDraw); }

      inline fun rowHeight() const noexcept -> u32 { return m_rowHeight; }
      inline fun setRowHeight(u32 val) -> void { m_rowHeight = val; update(visDirtyDraw); }
      qcl_prop(TYPE u32, NAME rowHeight, READ rowHeight, WRITE setRowHeight);
      
      inline fun headerHeight() const noexcept -> u32 { return m_headerHeight; }
      inline fun setHeaderHeight(u32 val) -> void { m_headerHeight = val; update(visDirtyDraw); }
      qcl_prop(TYPE u32, NAME headerHeight, READ headerHeight, WRITE setHeaderHeight);
      
      inline fun lineColor() -> dcolor& { return m_lineColor; }


    public:
      qcl_func qev<void, i32 /* Row */, i32 /* Col */> onCellClick;

    public:
      fun draw(renderContext&) -> void override;

      fun calcAutoSize(renderContext&) -> void override;

      fun doReset(renderContext&) -> void override;
      fun doClickEx(poit<f32> Pos) -> void override;
      virtual fun doCellClick(i32 Row, i32 Col) -> void;

    public:
      fun getProp(string_view) -> expected<any, bool> override;
      fun setProp(string_view, any) -> expected<void, bool> override;
      fun loadProp(string_view, const ds::value&, function<expected<qev_seed,bool>(string_view)>) -> expected<bool, string> override;
  };


  struct qcl_object popupMenu: qcl::contextMenu
  {
    public:
      struct popup_item
      {
        public:
          ~popup_item()
          {
            for (auto &X: m_items) delete X;
            m_items.clear();
          }


        private:
          string m_name;

          popup_item *m_parent{};
          vector<popup_item*> m_items;


        public:
          inline fun name() const noexcept -> string {return m_name;}
          inline fun setName(string_view Val) noexcept -> void {m_name = Val;}

          inline fun parent() const noexcept -> popup_item* {return m_parent;}
          inline fun setParent(popup_item* Val) noexcept -> void {m_parent = Val;}

          inline fun items() const noexcept -> vector<popup_item*> {return m_items;}

          inline fun addItem(popup_item* Val) noexcept -> void {m_items.push_back(Val);}
          inline fun remItem(popup_item* Val) noexcept -> void
          {
            delete relItem(Val);
          }
          inline fun relItem(popup_item* Val) noexcept -> popup_item*
          {
            auto it = std::find(m_items.begin(), m_items.end(), Val);

            if (it != m_items.end())
            {
              auto Ret = *it;
              m_items.erase(it);
              return Ret;
            }

            return Nil;
          }


        public:
          qev<void> onClick;
      };

    private:
      popup_item m_items;
      popup_item *m_nowItems{&m_items};
      i32 m_hSel{-1};

      rect<i32> PR = {3,3,6,6};

    public:
      fun propItems(const ds::value&, function<expected<qev_seed,bool>(string_view)>) -> expected<bool, string>;
      qcl_prop(TYPE string, NAME items, PROP FUNC propItems);


    public:
      fun showPopup(poit<i32>) -> void override;

      fun draw(renderContext&) -> void override;

      fun doMouseUp(poit<f32> Pos, shiftStateSet Button, shiftStateSet State) -> void override;
      fun doMouseMove(poit<f32> Pos, shiftStateSet State) -> void override;
    
      fun handlerStateChanged(qcl::visual::stateFlagSet State) -> void override;

    public:
      fun getProp(string_view) -> expected<any, bool> override;
      fun setProp(string_view, any) -> expected<void, bool> override;
      fun loadProp(string_view, const ds::value&, function<expected<qev_seed,bool>(string_view)>) -> expected<bool, string> override;
  };



  struct qcl_object chip: qcl::view
  {
    private:
      dcolor m_color{this, monetRole::SurfaceBright};
      f32 m_borderRadius{-1};

    public:
      inline fun color() -> dcolor& { return m_color; }
      inline fun propColor(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isString()) return std::unexpected("was expected string");

        m_color.update(Prop.w_string());

        return true;
      }
      qcl_prop(TYPE color, NAME color, PROP propColor);


      inline fun borderRadius() const noexcept -> f32 { return m_borderRadius; }
      inline fun setBorderRadius(f32 val) -> void { m_borderRadius = val; update(visDirtyDraw); }
      inline fun propBorderRadius(const ds::value& Prop) -> std::expected<bool, std::string> {
        
        if (Prop.isFloat())
          setBorderRadius(Prop.w_float());
        ef (Prop.isInt())
          setBorderRadius(Prop.w_int());
        else
          return std::unexpected("was expected float or int");

        return true;
      }
      qcl_prop(TYPE f32, NAME borderRadius, READ borderRadius, WRITE setBorderRadius, PROP propBorderRadius);

    public:
      fun draw(renderContext&) -> void override;
      fun doReset(renderContext&) -> void override;

    public:
      fun getProp(string_view) -> expected<any, bool> override;
      fun setProp(string_view, any) -> expected<void, bool> override;
      fun loadProp(string_view, const ds::value&, function<expected<qev_seed,bool>(string_view)>) -> expected<bool, string> override;
  };

  struct qcl_object card: qcl::view
  {
    private:
      dcolor m_color{this, monetRole::SurfaceBright};
      f32 m_borderRadius{-1};

    public:
      inline fun color() -> dcolor& { return m_color; }
      inline fun propColor(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isString()) return std::unexpected("was expected string");

        m_color.update(Prop.w_string());

        return true;
      }
      qcl_prop(TYPE color, NAME color, PROP propColor);
      

      inline fun borderRadius() const noexcept -> f32 { return m_borderRadius; }
      inline fun setBorderRadius(f32 val) -> void { m_borderRadius = val; update(visDirtyDraw); }
      inline fun propBorderRadius(const ds::value& Prop) -> std::expected<bool, std::string> {
        
        if (Prop.isFloat())
          setBorderRadius(Prop.w_float());
        ef (Prop.isInt())
          setBorderRadius(Prop.w_int());
        else
          return std::unexpected("was expected float or int");

        return true;
      }
      qcl_prop(TYPE f32, NAME borderRadius, READ borderRadius, WRITE setBorderRadius, PROP propBorderRadius);

    public:
      fun draw(renderContext&) -> void override;
      fun doReset(renderContext&) -> void override;

    public:
      fun getProp(string_view) -> expected<any, bool> override;
      fun setProp(string_view, any) -> expected<void, bool> override;
      fun loadProp(string_view, const ds::value&, function<expected<qev_seed,bool>(string_view)>) -> expected<bool, string> override;
  };
  
}
