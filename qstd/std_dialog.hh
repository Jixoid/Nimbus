/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


#pragma once

#include "include/core/SkFont.h"
#include "include/core/SkFontMetrics.h"
#include "include/core/SkRRect.h"
#include "qcl/platform.hh"
#include "qcl/view.hh"
#include "qcl/widget.hh"
#include "qcl/window.hh"
#include "qstd/monet.hh"

using namespace qcl;



namespace qstd
{

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
          break;
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

}
