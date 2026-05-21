/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


#pragma once

#include "qcl/ds/ds.hh"
#include "qcl/widget.hh"
#include "qstd/monet.hh"

using namespace qcl;



namespace qstd
{

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
      fun getProp(string_view) -> expected<std::any, bool> override;
      fun setProp(string_view, std::any) -> expected<void, bool> override;
      fun loadProp(string_view, const ds::value&, function<expected<qev_seed,bool>(string_view)>) -> expected<bool, string> override;
  };

}
