/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/



#include "DS.hh"
#include "VFS.hh"
#include "include/core/SkBlendMode.h"
#include "include/core/SkClipOp.h"
#include "include/core/SkColor.h"
#include "include/core/SkColorFilter.h"
#include "include/core/SkColorType.h"
#include "include/core/SkFontTypes.h"
#include "include/core/SkSamplingOptions.h"
#include "include/core/SkSize.h"
#include "include/private/base/SkPoint_impl.h"
#include "modules/svg/include/SkSVGDOM.h"
#include "include/core/SkStream.h"
#include "include/core/SkBitmap.h"
#include "include/core/SkSurface.h"
#include "include/core/SkCanvas.h"
#include "include/core/SkFont.h"
#include "include/core/SkPaint.h"
#include "include/core/SkFontMetrics.h"
#include "include/core/SkPath.h"
#include "include/core/SkPathBuilder.h"
#include "include/core/SkRRect.h"
#include "include/core/SkRect.h"
#include "qcl/Object.hh"
#include "qcl/Popup.hh"
#include "qcl/Types.hh"
#include "qcl/View.hh"
#include "qcl/Visual.hh"
#include "qcl/Window.hh"
#include "qstd/Monet.hh"
#include <expected>
#include <string>
#include <string_view>
#define el else
#define ef else if

#include <iostream>
#include <vector>

#include "Basis.h"

#include "qcl/Widget.hh"
#include "qcl/Application.hh"
#include "qcl/DynamicLoad.hh"
#include "qcl/Effect.hh"
#include "qstd/Standard.hh"

using namespace std;
using namespace qcl;


#include "qstd/std_multiprogbar.hh"
#include "qstd/std_interval.hh"

namespace qstd
{
  #define regO(X) {#X, {[]() -> object* { return new X(); }, [](object* val) -> bool { return dynamic_cast<X*>(val); }}}

  const dyn::objs _objs = {
    {"empty", {[]() -> object* { return new qcl::widget; }, [](object* val) -> bool { return dynamic_cast<qcl::widget*>(val); }}},
    regO(form),
    regO(layout),
    regO(layout_vert),
    regO(layout_horz),
    regO(layout_flow),
    regO(text),
    regO(icon),
    regO(edit),
    regO(radio),
    regO(check),
    regO(toggle),
    regO(progbar),
    regO(slider),
    regO(tabs),
    regO(choice),
    regO(grid),
    regO(popupMenu),
    regO(chip),
    regO(card),

    regO(multi_progbar),
    regO(interval),
  };

  #undef regO

  fun objs() -> dyn::objs { return _objs; }


  

  #pragma region form

  fun form::draw(renderContext& Context) -> void
  {
    auto Surface = Context.canvas();
    SkPaint Paint(Context.paint());
    
    Paint.setColor4f(m_color);

    Surface->drawRect({0,0, (f32)size().W, (f32)size().H}, Paint);
  }

  fun form::doReset(renderContext& Ctx) -> void
  {
    m_color.update();

    view::doReset(Ctx);
  }

  #pragma endregion



  #pragma region layout

  fun layout::draw(renderContext& Context) -> void
  {
    auto Surface = Context.canvas();
    SkPaint Paint(Context.paint());

    Paint.setColor4f(m_color);
    for (auto &X: effects()) if (X->enabled()) X->modify(Paint);
    
    Surface->drawRRect(SkRRect::MakeRectXY({0,0, (f32)size().W, (f32)size().H}, m_borderRadius, m_borderRadius), Paint);
  }

  fun layout::draw_scrollVert(renderContext& Context) -> void
  {
    if (!scrollVertVisible())
      return;

    auto Surface = Context.canvas();
    SkPaint Paint(Context.paint());

    #warning "Fix Me (RoundRect)"
    Surface->save();
    Surface->clipRect({0,0, (f32)size().W, (f32)size().H});


    Paint.setStrokeWidth(6);

    Paint.setColor4f(Monet.get(monetRole::SurfaceDim) *0.4);
    Surface->drawLine({(f32)size().W -3,0}, {(f32)size().W -3, (f32)size().H}, Paint);

    f32
      MaxSize = (clientBound().Y2-clientBound().Y1),
      Pos = -clientPos().Y,
      Height = height();

    Paint.setColor4f(Monet.get(monetRole::Primary));
    Surface->drawLine(
      {(f32)size().W -3, ((Pos /MaxSize) *Height)},
      {(f32)size().W -3, ((Pos /MaxSize) *Height) +((Height /MaxSize) *Height)},
      Paint
    );
    

    Surface->restore();
  }

  fun layout::draw_scrollHorz(renderContext& Context) -> void
  {
    if (!scrollHorzVisible())
      return;

    auto Surface = Context.canvas();
    SkPaint Paint(Context.paint());


    #warning "Fix Me (Round Rect)"
    Surface->save();
    Surface->clipRect({0,0, (f32)size().W, (f32)size().H});


    Paint.setStrokeWidth(6);

    Paint.setColor4f(Monet.get(monetRole::SurfaceDim) *0.4);
    Surface->drawLine({0, (f32)size().H -3}, {(f32)size().W, (f32)size().H -3}, Paint);

    f32
      MaxSize = (clientBound().X2-clientBound().X1),
      Pos = -clientPos().X,
      Width = width();

    Paint.setColor4f(Monet.get(monetRole::Primary));
    Surface->drawLine(
      {((Pos /MaxSize) *Width), (f32)size().H -3},
      {((Pos /MaxSize) *Width) +((Width /MaxSize) *Width), (f32)size().H -3},
      Paint
    );


    Surface->restore();
  }

  fun layout::doReset(renderContext& Ctx) -> void
  {
    m_color.update();

    view::doReset(Ctx);
  }

  #pragma endregion


  #pragma region layout_vert

  fun layout_vert::doTiling(renderContext& Context) -> void
  {
    i32
      StartPos{},
      LastMargin = paddings().top();

    // Tiling
    for (auto &X: widgets())
    {
      StartPos += (X->margins().top() > LastMargin) ? (X->margins().top()):(LastMargin);

      // End Pos
      auto NSize = qcl::size<i32>(
        X->canAutoSizeHorz() ? X->preferedSize().W : X->size().W,
        X->canAutoSizeVert() ? X->preferedSize().H : X->size().H
      );

      NSize.W = max(X->minSize().W, NSize.W);
      NSize.H = max(X->minSize().H, NSize.H);

      if (X->maxSize().W != 0) NSize.W = min(X->maxSize().W, NSize.W);
      if (X->maxSize().H != 0) NSize.H = min(X->maxSize().H, NSize.H);

      i32
        SPos = std::max(paddings().left(), X->margins().left()),
        EPos = std::min(SPos +NSize.W, width() -paddings().right());


      // End Fixed
      if (X->anchors().right().active()) EPos = width() -X->margins().right() -paddings().right();
      if (!X->anchors().left().active() && X->anchors().right().active()) SPos = EPos -NSize.W;

      if (X->point().X != SPos || X->point().Y != StartPos)
        X->setPoint(poit<i32>(SPos, StartPos));

      if (X->width() != (EPos -SPos) || X->height() != NSize.H) {
        X->setSize(qcl::size<i32>(EPos -SPos, NSize.H));

        X->setWidth( max(X->minSize().W, X->width()) );
        X->setHeight( max(X->minSize().H, X->height()) );

        if (X->maxSize().W != 0) X->setWidth( min(X->maxSize().W, X->width()) );
        if (X->maxSize().H != 0) X->setHeight( min(X->maxSize().H, X->height()) );
      }


      X->doPaint_prepare(Context);

      // Tiling
      LastMargin = X->margins().bottom();
      StartPos += X->height();
    }


    // Others
    fixOverScroll();
  }

  #pragma endregion


  #pragma region layout_horz

  fun layout_horz::doTiling(renderContext& Context) -> void
  {
    i32
      StartPos = 0,
      LastMargin = 0;

    // Tiling
    for (auto &X: widgets())
    {
      StartPos += (X->margins().left() > LastMargin) ? (X->margins().left()):(LastMargin);

      // End Pos
      auto NSize = qcl::size<i32>(
        X->canAutoSizeHorz() ? X->preferedSize().W : X->size().W,
        X->canAutoSizeVert() ? X->preferedSize().H : X->size().H
      );

      NSize.W = max(X->minSize().W, NSize.W);
      NSize.H = max(X->minSize().H, NSize.H);

      if (X->maxSize().W != 0) NSize.W = min(X->maxSize().W, NSize.W);
      if (X->maxSize().H != 0) NSize.H = min(X->maxSize().H, NSize.H);

      i32
        SPos = X->margins().top(),
        EPos = SPos +NSize.H;


      // End Fixed
      if (X->anchors().bottom().active()) EPos = height() -X->margins().bottom();
      if (!X->anchors().top().active() && X->anchors().bottom().active()) SPos = EPos -NSize.H;

      if (X->point().Y != SPos || X->point().X != StartPos)
        X->setPoint({StartPos, SPos});

      if (X->height() != (EPos -SPos) || X->width() != NSize.W)
      {
        X->setSize({NSize.W, EPos -SPos});

        X->setWidth( max(X->minSize().W, X->width()) );
        X->setHeight( max(X->minSize().H, X->height()) );

        if (X->maxSize().W != 0) X->setWidth( min(X->maxSize().W, X->width()) );
        if (X->maxSize().H != 0) X->setHeight( min(X->maxSize().H, X->height()) );
      }


      X->doPaint_prepare(Context);

      // Tiling
      LastMargin = X->margins().right();
      StartPos += X->width();
    }


    // Others
    fixOverScroll();
  }

  #pragma endregion


  #pragma region layout_flow

  fun layout_flow::doTiling(renderContext& Context) -> void
  {
    poit<i32> CPos{}, LPad{}; // R,B
    i32 LineHeight{}, LastMargin{};

    // Tiling
    for (auto &X: widgets())
    {
      if (!X->visible())
        continue;

      auto NSize = qcl::size<i32>(
        X->canAutoSizeHorz() ? X->preferedSize().W : X->size().W,
        X->canAutoSizeVert() ? X->preferedSize().H : X->size().H
      );

      NSize.W = max(X->minSize().W, NSize.W);
      NSize.H = max(X->minSize().H, NSize.H);

      if (X->maxSize().W != 0) NSize.W = min(X->maxSize().W, NSize.W);
      if (X->maxSize().H != 0) NSize.H = min(X->maxSize().H, NSize.H);

      
      i32 RequiredWidth = X->margins().left() +NSize.W +X->margins().right() +LPad.X;
      
      if (CPos.X +RequiredWidth > width() && CPos.X > 0)
      {
        CPos.X = 0;
        CPos.Y += LineHeight + LPad.Y;
        LineHeight = 0;
        LastMargin = 0;
      }

      i32 HorzMargin = (X->margins().left() > LastMargin) ? X->margins().left() : LastMargin;
      i32 TargetX = CPos.X + HorzMargin;
      i32 TargetY = CPos.Y + X->margins().top();

      i32 TargetH = NSize.H;
      if (X->anchors().bottom().active() && LineHeight > 0) 
        TargetH = max(NSize.H, LineHeight - X->margins().top() - X->margins().bottom());
      
      if (X->point().Y != TargetY || X->point().X != TargetX)
        X->setPoint({TargetX, TargetY});

      if (X->width() != NSize.W || X->height() != TargetH)
      {
        X->setSize({NSize.W, TargetH});
        
        X->setWidth( max(X->minSize().W, X->width()) );
        X->setHeight( max(X->minSize().H, X->height()) );
        if (X->maxSize().W != 0) X->setWidth( min(X->maxSize().W, X->width()) );
        if (X->maxSize().H != 0) X->setHeight( min(X->maxSize().H, X->height()) );
      }


      X->doPaint_prepare(Context);


      LastMargin = X->margins().right();
      CPos.X = X->point().X + X->width();

      i32 TotalItemHeight = X->height() + X->margins().top() + X->margins().bottom();
      if (TotalItemHeight > LineHeight)
          LineHeight = TotalItemHeight;
    }


    // Others
    fixOverScroll();
  }

  #pragma endregion


  #pragma region dialog

  fun dialog_titleBar::setButtons() -> void
  {
    m_btnClose->setParent(this);
    m_btnClose->setButtonType(dialog_titleButtonType::dtbtClose);
    m_btnClose->anchors().top().set(true, this, widgetAnchorSide::wasBeg);
    m_btnClose->anchors().right().set(true, this, widgetAnchorSide::wasEnd);
    m_btnClose->anchors().bottom().set(true, this, widgetAnchorSide::wasEnd);
    m_btnClose->margins().set({0,5,5,5});
  }

  fun dialog::setTitleBar() -> void
  {
    m_titleBar->setParent(this);
    m_titleBar->anchors().top().set(true, this, widgetAnchorSide::wasBeg);
    m_titleBar->anchors().left().set(true, this, widgetAnchorSide::wasBeg);
    m_titleBar->anchors().right().set(true, this, widgetAnchorSide::wasEnd);
    m_titleBar->margins().set({12,12,12,0});
  }

  fun dialog::showModal() -> void
  {
    if (auto win = dynamic_cast<window*>(getRoot()))
    {
      setPoint({(win->width()-width())/2, (win->height()-height())/2});

      win->setOverlay(this, windowOverlayMode::womShadow);
    }
  }

  fun dialog::draw(renderContext& Context) -> void
  {
    constexpr auto borderRadius = 16;


    auto Surface = Context.canvas();
    auto Paint = SkPaint(Context.paint());

    rect<i32> PR = {12,12,12,12};
    {
      auto rrect = SkRRect::MakeRectXY(SkRect::MakeLTRB(PR.X1,PR.Y1, (f32)width()-PR.X2, (f32)height()-PR.Y2), borderRadius,borderRadius);

      SkPaint shadowPaint;
      shadowPaint.setColor(SK_ColorBLACK);
      shadowPaint.setAlphaf(0.5);
      shadowPaint.setAntiAlias(true);

      float blurSigma = 4.0;
      shadowPaint.setMaskFilter(SkMaskFilter::MakeBlur(kNormal_SkBlurStyle, blurSigma));

      Surface->drawRRect(rrect, shadowPaint);
    }


    Paint.setColor4f(Monet.get(monetRole::SurfaceContainer));
    Surface->drawRRect(SkRRect::MakeRectXY(SkRect::MakeLTRB(PR.X1, PR.Y1, width()-PR.X2, height()-PR.Y2), borderRadius, borderRadius), Paint);
  }

  #pragma endregion


  #pragma region icon

  fun icon::draw(renderContext& Context) -> void
  {
    if (m_icon == Nil)
      return;

    auto Canvas = Context.canvas();

    auto Img = m_icon->makeImageSnapshot();

    Canvas->save();

    SkMatrix m = Canvas->getTotalMatrix();

    m.setScaleX(1.0);
    m.setScaleY(1.0);

    Canvas->setMatrix(m);

    SkPaint Paint;
    Paint.setColorFilter(SkColorFilters::Blend(SkColor4f(m_theme).toSkColor(), SkBlendMode::kSrcIn));

    Canvas->drawImage(Img, 0,0, SkSamplingOptions(), &Paint);
    
    Canvas->restore();
  }


  void PasteSvgOnCanvas(SkCanvas* targetCanvas, sk_sp<SkFontMgr> fontMgr, istream* svgStream, float x, float y, float targetWidth = 0)
  {
    // 1. std::istream'i verimli bir şekilde SkData'ya çevir
    // (SVG parse işlemi seek/rewind gerektirebileceği için buffer'a almak en güvenlisidir)
    std::vector<char> buffer((std::istreambuf_iterator<char>(*svgStream)),
                              std::istreambuf_iterator<char>());
    
    if (buffer.empty()) {
      std::cerr << "HATA: SVG akışı boş!" << std::endl;
      return;
    }

    auto data = SkData::MakeWithCopy(buffer.data(), buffer.size());
    auto skStream = SkMemoryStream::Make(data);

    // 2. SVG DOM Oluştur
    auto dom = SkSVGDOM::Builder().setFontManager(fontMgr).make(*skStream);
    if (!dom) {
      std::cerr << "HATA: SVG parse edilemedi!" << std::endl;
      return;
    }

    // 3. SVG'nin orijinal boyutlarını al
    // SVG'de width/height yoksa varsayılan bir boyut (örn: viewBox) döner.
    SkSize svgSize = dom->containerSize();
    if (svgSize.isEmpty()) {
      svgSize = SkSize::Make(100, 100); // Hiçbir boyut yoksa varsayılan
      dom->setContainerSize(svgSize);
    }

    // 4. Canvas üzerinde pozisyon ve ölçekleme ayarla
    // Canvas durumunu kaydet (işimiz bitince geri yüklemek için)
    targetCanvas->save();

    // İstenilen (x, y) konumuna git
    targetCanvas->translate(x, y);

    // Eğer bir genişlik hedefi verildiyse ölçekleme (scale) yap
    if (targetWidth > 0 && svgSize.width() > 0) {
      float scale = targetWidth / svgSize.width();
      targetCanvas->scale(scale, scale);
    }

    // 5. SVG'yi Canvas'a Render et
    // Not: render fonksiyonu her zaman (0,0) noktasına çizer,
    // bu yüzden yukarıda translate() kullandık.
    dom->render(targetCanvas);

    // Canvas ayarlarını (translate/scale) geri al
    targetCanvas->restore();
  }


  fun icon::doPaint_prepare(renderContext& Context) -> void
  {
    if (!m_reRender)
      widget::doPaint_prepare(Context);


    if (m_icon) m_icon = Nil;

    if (!m_path.empty())
    {
      auto Stream = vfs::Resolve_RO(m_path);
    
      auto ImgInfo = SkImageInfo::MakeN32Premul(Context.dp(width()), Context.dp(height()));
      m_icon = SkSurfaces::Raster(ImgInfo);

      auto Canv = m_icon->getCanvas();

      PasteSvgOnCanvas(Canv, Context.fontMgr(), Stream.get(), 0,0, Context.dp(width()));
    }

    m_reRender = false;
    

    widget::doPaint_prepare(Context);
  }

  #pragma endregion
  

  #pragma region radio

  fun radio::draw(renderContext& Context) -> void
  {
    auto Surface = Context.canvas();
    SkFont Font(Context.font());
    SkPaint Paint(Context.paint());

    color Theme;

    if ((stateFlags() & (sfHover)) != 0)
      Theme = Monet.get(monetRole::PrimaryContainer);
    else
      Theme = Monet.get(monetRole::SurfaceOutline);

    
    Paint.setColor4f(Theme);
    Paint.setStrokeWidth(2);
    Paint.setStyle(SkPaint::kStroke_Style);

    Surface->drawCircle({11, 11}, 10, Paint);


    if (m_checked) {
      Paint = SkPaint(Context.paint());
      Paint.setColor4f(Monet.get(monetRole::PrimaryContainer));
      Paint.setStyle(SkPaint::kFill_Style);
      for (auto &X: effects()) if (X->enabled()) X->modify(Paint);
      
      Surface->drawCircle({11,11},5, Paint);
    }
    
    
    Font.setSize(15);
    
    SkFontMetrics metrics;
    Font.getMetrics(&metrics);
    
    f32 Pos = (height() / 2.0f) - ((metrics.fAscent + metrics.fDescent) / 2.0f);
    
    Paint = SkPaint(Context.paint());
    Paint.setColor4f(Monet.get(monetRole::OnSurface));
    Paint.setStyle(SkPaint::kFill_Style);
    Surface->drawString(m_text.c_str(), 27, Pos, Font, Paint);
  }

  fun radio::handlerStateChanged(visual::stateFlagSet State) -> void
  {
    widget::handlerStateChanged(State);

    update(visDirtyDraw);
  }

  fun radio::doClick() -> void
  {
    widget::doClick();

    if (m_checked)
      return;

    if (parent() != Nil)
      for (auto &X: parent()->childs())
        if (auto C = dynamic_cast<qstd::radio*>(X); C != Nil)
          if (C->m_checked && C != this)
            C->doChanged(false);
    
    if (!m_checked)
      doChanged(true);
  }

  fun radio::doChanged(bool Status) -> void
  {
    m_checked = Status;

    onChanged(this, Status);

    update(visDirtyDraw);
  }

  fun radio::calcAutoSize(renderContext& Context) -> void
  {
    SkFont Font(Context.font());

    
    SkRect TSize;
    Font.measureText(m_text.c_str(), m_text.size(), SkTextEncoding::kUTF8, &TSize);
    
    setPreferedSize({(i32)TSize.width() +30 +8, max<i32>(TSize.height() +8, 22)});
  }

  #pragma endregion


  #pragma region check

  fun check::draw(renderContext& Context) -> void
  {
    auto Surface = Context.canvas();
    SkFont Font(Context.font());
    SkPaint Paint(Context.paint());

    color Theme;

    if ((stateFlags() & (sfHover)) != 0)
      Theme = Monet.get(monetRole::PrimaryContainer);
    else
      Theme = Monet.get(monetRole::SurfaceOutline);
      
    
    Paint.setColor4f(Theme);
    Paint.setStrokeWidth(2);
    Paint.setStyle(SkPaint::kStroke_Style);

    Surface->drawRoundRect(SkRect::MakeXYWH(1,1, 20, 20), 7, 7, Paint);

    if (m_checked) {
      Paint = SkPaint(Context.paint());
      Paint.setColor4f(Monet.get(monetRole::PrimaryContainer));
      Paint.setStyle(SkPaint::kFill_Style);
      for (auto &X: effects()) if (X->enabled()) X->modify(Paint);

      Surface->drawRoundRect(SkRect::MakeXYWH(11-5, 11-5, 10, 10), 3, 3, Paint);
    }


    Font.setSize(15);


    SkFontMetrics metrics;
    Font.getMetrics(&metrics);

    f32 Pos = (height() / 2.0f) - ((metrics.fAscent + metrics.fDescent) / 2.0f);

    Paint = SkPaint(Context.paint());
    Paint.setColor4f(Monet.get(monetRole::OnSurface));
    Paint.setStyle(SkPaint::kFill_Style);
    Surface->drawString(m_text.c_str(), 27, Pos, Font, Paint);
  }

  fun check::handlerStateChanged(visual::stateFlagSet State) -> void
  {
    widget::handlerStateChanged(State);

    update(visDirtyDraw);
  }

  fun check::doClick() -> void
  {
    widget::doClick();

    doChanged(!m_checked);
  }

  fun check::doChanged(bool Status) -> void
  {
    m_checked = Status;

    onChanged(this, Status);

    update(visDirtyDraw);
  }

  fun check::calcAutoSize(renderContext& Context) -> void
  {
    SkFont Font(Context.font());
    
    SkRect TSize;
    Font.measureText(m_text.c_str(), m_text.size(), SkTextEncoding::kUTF8, &TSize);
    
    setPreferedSize({(i32)TSize.width() +30 +8, max<i32>(TSize.height() +8, 22)});
  }

  #pragma endregion


  #pragma region toggle

  fun toggle::draw(renderContext& Context) -> void
  {
    auto Surface = Context.canvas();
    SkFont Font(Context.font());
    SkPaint Paint(Context.paint());
    
    monetRole Theme  = (stateFlags() & (sfHover)) != 0 ? monetRole::PrimaryContainer : monetRole::SurfaceOutline;
    monetRole Theme2 = (stateFlags() & (sfHover)) != 0 ? monetRole::PrimaryContainer : monetRole::PrimaryContainer;


    Paint.setStrokeWidth(2);

    if (!m_checked) {
      Paint.setColor4f(Monet.get(Theme));
      Paint.setStyle(SkPaint::kStroke_Style);

      Surface->drawRoundRect(SkRect::MakeXYWH((f32)size().W -50, 1, 49, 24), 12, 12, Paint);
      
      
      Paint = SkPaint(Context.paint());
      Paint.setColor4f(Monet.get(Theme2));
      Paint.setStyle(SkPaint::kFill_Style);
      for (auto &X: effects()) if (X->enabled()) X->modify(Paint);

      Surface->drawCircle((f32)size().W -38, 13, 7, Paint); 
    }
    else {
      Paint.setColor4f(Monet.get(monetRole::PrimaryContainer));
      Paint.setStyle(SkPaint::kStrokeAndFill_Style);
      for (auto &X: effects()) if (X->enabled()) X->modify(Paint);

      Surface->drawRoundRect(SkRect::MakeXYWH((f32)size().W -50, 1, 49, 24), 12, 12, Paint);

      Paint = SkPaint(Context.paint());
      Paint.setColor4f(Monet.get(monetRole::SurfaceContainerLow) *0.9);
      Surface->drawCircle((f32)size().W -13, 13, 9, Paint);
    }


    Font.setSize(15);

    SkFontMetrics metrics;
    Font.getMetrics(&metrics);

    f32 Pos = (height() / 2.0f) - ((metrics.fAscent + metrics.fDescent) / 2.0f);

    Paint = SkPaint(Context.paint());
    Paint.setStyle(SkPaint::kFill_Style);
    Paint.setColor4f(Monet.get(monetRole::OnSurface));
    Surface->drawString(m_text.c_str(), 0, Pos, Font, Paint);
  }

  fun toggle::handlerStateChanged(visual::stateFlagSet State) -> void
  {
    widget::handlerStateChanged(State);

    update(visDirtyDraw);
  }

  fun toggle::doClick() -> void
  {
    widget::doClick();
    
    doChanged(!m_checked);
  }

  fun toggle::doChanged(bool Status) -> void
  {
    m_checked = Status;

    onChanged(this, Status);


    update(visDirtyDraw);
  }

  fun toggle::calcAutoSize(renderContext& Context) -> void
  {
    SkFont Font(Context.font());

    SkFontMetrics metrics;
    Font.getMetrics(&metrics);

    f32 Pos = (height() / 2.0f) - ((metrics.fAscent + metrics.fDescent) / 2.0f);
    
    SkRect TSize;
    Font.measureText(m_text.c_str(), m_text.size(), SkTextEncoding::kUTF8, &TSize);

    setPreferedSize({(i32)TSize.width() +8 +70, max<i32>(Pos +8, 26)});
  }

  #pragma endregion


  #pragma region progbar

  fun progbar::draw(renderContext& Context) -> void
  {
    auto Surface = Context.canvas();
    SkFont Font(Context.font());
    SkPaint Paint(Context.paint());

    Paint.setStrokeWidth(2);

    SkRRect RRect;
    RRect.setRectXY(SkRect::MakeXYWH(0,0, (f32)size().W, (f32)size().H), 14, 14);

    Surface->save();
    Surface->clipRRect(RRect, true);

    Paint.setColor4f(Monet.get(monetRole::SurfaceContainerHigh));
    Paint.setStyle(SkPaint::kFill_Style);

    Surface->drawRect(SkRect::MakeXYWH(0,0, (f32)size().W, (f32)size().H), Paint);
    
    
    f32 ProgressW = ((f32)size().W / m_max * m_value);
    if (ProgressW > 0) {
      Paint = SkPaint(Context.paint());
      Paint.setColor4f(Monet.get(monetRole::PrimaryContainer));
      for (auto &X: effects()) if (X->enabled()) X->modify(Paint);

      Surface->drawRRect(SkRRect::MakeRectXY(SkRect::MakeXYWH(0, 0, ProgressW, (f32)size().H), 5,5), Paint);
    }
    
    Surface->restore(); // Restore main clip


    // Text Drawing
    Font.setSize(15);

    SkFontMetrics metrics;
    Font.getMetrics(&metrics);

    f32 Pos = (height() / 2.0f) - ((metrics.fAscent + metrics.fDescent) / 2.0f);

    SkRect TSize;
    Font.measureText(m_text.c_str(), m_text.size(), SkTextEncoding::kUTF8, &TSize);

    f32 TX = ((size().W -TSize.width())/2);

    SkRRect ProgRRect;
    ProgRRect.setRectXY(SkRect::MakeLTRB(ProgressW,0, (f32)width(), (f32)height()), 14, 14);
    
    Surface->save();
    Surface->clipRRect(ProgRRect, true);
    
    Paint = SkPaint(Context.paint());
    Paint.setColor4f(Monet.get(monetRole::OnSurfaceContainerHigh));
    Paint.setStyle(SkPaint::kFill_Style);
    Surface->drawString(m_text.c_str(), TX, Pos, Font, Paint);

    Surface->restore();

    
    if (ProgressW > 0) {
      SkRRect ProgRRect;
      ProgRRect.setRectXY(SkRect::MakeXYWH(0,0, ProgressW, (f32)size().H), 14, 14);
      
      Surface->save();
      Surface->clipRRect(ProgRRect, true);

      if (!m_font)
        m_font = Context.fontType(0.7, 0, -0.4,0);
      
      Paint = SkPaint(Context.paint());
      Paint.setColor4f(Monet.get(monetRole::OnPrimaryContainer));
      Surface->drawString(m_text.c_str(), TX, Pos, SkFont(m_font, 15), Paint);
      
      Surface->restore();
    }
  }

  #pragma endregion


  #pragma region slider

  fun slider::draw(renderContext& Context) -> void
  {
    auto Surface = Context.canvas();
    SkPaint Paint(Context.paint());

    Paint.setStrokeWidth(2);

    // Common Calc
    f32 W = (f32)size().W;
    f32 H = (f32)size().H;
    f32 ValPos = ((W - 4.0f) / m_max * m_value) + 2.0f;

    if ((stateFlags() & (sfHover)) != 0)
    {
      // Filled (Left)
      SkVector RadiiLeft[4] = { {14,14}, {4,4}, {4,4}, {14,14} }; // TL, TR, BR, BL
      SkRRect RRectLeft;
      RRectLeft.setRectRadii(SkRect::MakeLTRB(1, 4, ValPos-7, H-4), RadiiLeft);
      
      Paint.setColor4f(Monet.get(monetRole::SecondaryContainer));
      Paint.setStyle(SkPaint::kFill_Style);
      for (auto &X: effects()) if (X->enabled()) X->modify(Paint);

      Surface->drawRRect(RRectLeft, Paint);


      // Handle (Middle)
      Paint = SkPaint(Context.paint());
      Paint.setColor4f(Monet.get(monetRole::PrimaryContainer));
      for (auto &X: effects()) if (X->enabled()) X->modify(Paint);

      Surface->drawRoundRect(SkRect::MakeLTRB(ValPos-3, 1, ValPos+3, H-1), 4, 4, Paint);


      // Empty Part (Right)-
      SkVector RadiiRight[4] = {{4,4}, {14,14}, {14,14}, {4,4}};
      SkRRect RRectRight;
      RRectRight.setRectRadii(SkRect::MakeLTRB(ValPos+7, 4, W-1, H-4), RadiiRight);
      
      Paint = SkPaint(Context.paint());
      Paint.setColor4f(Monet.get(monetRole::SurfaceContainerHigh) *0.5);
      
      Surface->drawRRect(RRectRight, Paint);
    }
      
    else
    {
      // Filled (Left)
      SkVector RadiiLeft[4] = {{14,14}, {4,4}, {4,4}, {14,14}};
      SkRRect RRectLeft;
      RRectLeft.setRectRadii(SkRect::MakeLTRB(1, 4, ValPos-6, H-4), RadiiLeft);

      
      Paint.setColor4f(Monet.get(monetRole::SecondaryContainer));
      Paint.setStyle(SkPaint::kFill_Style);
      for (auto &X: effects()) if (X->enabled()) X->modify(Paint);

      Surface->drawRRect(RRectLeft, Paint);


      // Handle (Middle)
      Paint = SkPaint(Context.paint());
      Paint.setColor4f(Monet.get(monetRole::PrimaryContainer));
      for (auto &X: effects()) if (X->enabled()) X->modify(Paint);

      Surface->drawRoundRect(SkRect::MakeLTRB(ValPos-2, 1, ValPos+2, H-1), 4, 4, Paint);


      // Empty Part (Right)
      SkVector RadiiRight[4] = {{4,4}, {14,14}, {14,14}, {4,4}};
      SkRRect RRectRight;
      RRectRight.setRectRadii(SkRect::MakeLTRB(ValPos+6, 4, W-1, H-4), RadiiRight);
      
      Paint = SkPaint(Context.paint());
      Paint.setColor4f(Monet.get(monetRole::SurfaceContainerHigh) *0.5);
      
      Surface->drawRRect(RRectRight, Paint);
    }
  }

  fun slider::handlerStateChanged(visual::stateFlagSet State) -> void
  {
    widget::handlerStateChanged(State);

    update(visDirtyDraw);
  }

  fun slider::doMouseDown(poit<f32> Pos, shiftStateSet Button, shiftStateSet State) -> void
  {
    if (Button == shiftStates::ssLeft)
    {
      i32 Step = min<i32>(size().W, std::max<i32>(0, (i32)Pos.X));
      
      doChanged((f32)Step /(f32)size().W *m_max);
    }

    widget::doMouseDown(Pos, Button, State);
  }

  fun slider::doMouseMove(poit<f32> Pos, shiftStateSet State) -> void
  {
    if ((State & shiftStates::ssLeft) != 0)
    {
      i32 Step = min<i32>(size().W, std::max<i32>(0, Pos.X));

      doChanged((f32)Step /(f32)size().W *m_max);
    }

    widget::doMouseMove(Pos, State);
  }

  fun slider::doChanged(u32 nValue) -> void
  {
    m_value = nValue;

    onChanged(this, nValue);


    update(visDirtyDraw);
  }

  #pragma endregion



  #pragma region tabs

  fun tabs::draw(renderContext& Context) -> void
  {
    auto Surface = Context.canvas();
    SkFont Font(Context.font());
    SkPaint Paint(Context.paint());

    Paint.setStrokeWidth(2);
    Font.setSize(14);


    // Background
    SkRRect RRect;
    RRect.setRectXY(SkRect::MakeXYWH(0, 0, (f32)size().W, (f32)size().H), 18, 18);
    Surface->save();
    Surface->clipRRect(RRect, true);


    Paint.setColor4f(Monet.get(monetRole::SurfaceContainerHigh));
    Paint.setStyle(SkPaint::kFill_Style);
    Surface->drawRect(SkRect::MakeXYWH(0,0, (f32)size().W, (f32)size().H), Paint); 
    
    // Ensure m_optTabs is up to date
    if (m_optTabs.size() != m_tabs.size()) {
      m_optTabs.clear();
      for(const auto& t : m_tabs) {
        SkRect b;
        Font.measureText(t.c_str(), t.size(), SkTextEncoding::kUTF8, &b);
        m_optTabs.push_back({b.width(), b.height()});
      }
    }


    u32 XOff = 0;

    for (u32 i = 0; i < m_tabs.size(); i++)
    {
      if (i >= m_optTabs.size()) break; // Safety
      
      auto &X = m_tabs[i];
      XOff += 16;

      SkFontMetrics metrics;
      Font.getMetrics(&metrics);


      if (m_tabID == i)
      {
        Surface->save();

        SkRect TabRect = SkRect::MakeLTRB(
          (f32)XOff -6, 
          ((f32)size().H/2) -(m_optTabs[i].H/2) -6, 
          XOff +m_optTabs[i].W +6, 
          ((f32)size().H/2) +(m_optTabs[i].H/2) +6
        );

        SkRRect TabRRect;
        TabRRect.setRectXY(TabRect, 18, 18);
        Surface->clipRRect(TabRRect, true);

        Paint = SkPaint(Context.paint());
        Paint.setColor4f(Monet.get(monetRole::PrimaryContainer));
        Paint.setStyle(SkPaint::kFill_Style);
        for (auto &X: effects()) if (X->enabled()) X->modify(Paint);
        
        Surface->drawRect(TabRect, Paint);


        f32 Pos = (height() / 2.0f) - ((metrics.fAscent + metrics.fDescent) / 2.0f);

        Paint = SkPaint(Context.paint());
        Paint.setColor4f(Monet.get(monetRole::OnPrimaryContainer));
        Surface->drawString(X.c_str(), (f32)XOff, Pos, Font, Paint);

        Surface->restore(); // popSets
      }

      el {
        f32 Pos = (height() / 2.0f) - ((metrics.fAscent + metrics.fDescent) / 2.0f);
        
        Paint = SkPaint(Context.paint());
        Paint.setColor4f(Monet.get(monetRole::OnSurface));
        Surface->drawString(X.c_str(), (f32)XOff, Pos, Font, Paint);
      }

      
      XOff += m_optTabs[i].W;
    }


    Surface->restore(); // clip_Reset (Main Clip)
  }

  fun tabs::handlerStateChanged(visual::stateFlagSet State) -> void
  {
    widget::handlerStateChanged(State);

    m_hTabID = -1;
    update(visDirtyDraw);
  }

  fun tabs::recalcTabsSize(renderContext& Context) -> void
  {
    auto Font(Context.font());
    Font.setSize(14);
    
    
    m_optTabs.clear();
    m_optTabs.reserve(m_tabs.size());
    
    for (auto &X: m_tabs)
    {
      SkRect TSize;
      Font.measureText(X.c_str(), X.size(), SkTextEncoding::kUTF8, &TSize);

      m_optTabs.push_back({TSize.width(), TSize.height()});
    }
  }

  fun tabs::doMouseMove(poit<f32> Pos, shiftStateSet State) -> void
  {
    u32 XOff = 0;

    i32 NHTabID = -1;

    for (u32 i = 0; i < m_optTabs.size(); i++)
    {
      XOff += 16;

      if (
        Pos.X > XOff -3 &&
        Pos.X < XOff +m_optTabs[i].W +3
      )
      {
        NHTabID = i;
        break;
      }

      XOff += m_optTabs[i].W;
    }


    if (NHTabID != m_hTabID)
    {
      m_hTabID = NHTabID;

      update(visDirtyDraw);
    }


    widget::doMouseMove(Pos, State);
  }

  fun tabs::doClickEx(poit<f32> Pos) -> void
  {
    u32 XOff = 0;

    i32 NTabID = -1;

    for (u32 i = 0; i < m_optTabs.size(); i++)
    {
      XOff += 16;

      if (
        Pos.X > XOff -3 &&
        Pos.X < XOff +m_optTabs[i].W +3
      )
      {
        NTabID = i;
        break;
      }

      XOff += m_optTabs[i].W;
    }


    if (NTabID != -1 && NTabID != m_tabID)
      doChanged(NTabID);


    widget::doClickEx(Pos);
  }

  fun tabs::doChanged(i32 nTabID) -> void
  {
    m_tabID = nTabID;

    onChanged(this, nTabID);


    update(visDirtyDraw);
  }

  fun tabs::calcAutoSize(renderContext& Context) -> void
  {
    recalcTabsSize(Context);

    SkFont Font(Context.font());
    Font.setSize(14);
    
    auto XOff = 16;
    for (const auto& X : m_tabs) {
      SkRect b;
      Font.measureText(X.c_str(), X.size(), SkTextEncoding::kUTF8, &b);
      XOff += b.width() + 16;
    }
    
    setPreferedSize({XOff, 32});
  }

  #pragma endregion


  #pragma region choice

  fun choice::draw(renderContext& Context) -> void
  {
    auto Surface = Context.canvas();
    SkFont Font(Context.font());
    SkPaint Paint(Context.paint());
    
    Paint.setStrokeWidth(2);
    Font.setSize(14);


    // Background
    Paint.setColor4f(Monet.get(monetRole::SurfaceContainerHigh));
    Paint.setStyle(SkPaint::kFill_Style);

    Surface->drawRoundRect(SkRect::MakeLTRB(0, (f32)m_threshold, (f32)size().W, (f32)size().H -m_threshold), 18, 18, Paint);

    // Measure Loop if needed
    if (m_optTabs.size() != m_tabs.size()) {
      m_optTabs.clear();
      for(const auto& t : m_tabs) {
        SkRect b;
        Font.measureText(t.c_str(), t.size(), SkTextEncoding::kUTF8, &b);
        m_optTabs.push_back({b.width(), b.height()});
      }
    }


    i32 XOff = -10;

    for (u32 i = 0; i < m_tabs.size(); i++)
    {
      if (i >= m_optTabs.size()) break;

      auto &X = m_tabs[i];

      XOff += 16;


      SkFontMetrics metrics;
      Font.getMetrics(&metrics);

      if (m_tabID == i)
      {
        SkRect TabRect = SkRect::MakeLTRB(
          (f32)XOff -6, 
          0, 
          XOff +m_optTabs[i].W +6, 
          (f32)size().H
        );

        // Clip
        SkRRect TabRRect;
        TabRRect.setRectXY(TabRect, 18, 18);
        Surface->save();
        Surface->clipRRect(TabRRect, true);


        Paint = SkPaint(Context.paint());
        Paint.setColor4f(Monet.get(monetRole::PrimaryContainer));
        Paint.setStyle(SkPaint::kFill_Style);
        for (auto &X: effects()) if (X->enabled()) X->modify(Paint);
        
        Surface->drawRect(TabRect, Paint);

        f32 Pos = (height() / 2.0f) - ((metrics.fAscent + metrics.fDescent) / 2.0f);

        Paint = SkPaint(Context.paint());
        Paint.setColor4f(Monet.get(monetRole::OnPrimaryContainer));
        Surface->drawString(X.c_str(), (f32)XOff, Pos, Font, Paint);

        Surface->restore();
      }

      el {
        f32 Pos = (height() / 2.0f) - ((metrics.fAscent + metrics.fDescent) / 2.0f);
        
        Paint = SkPaint(Context.paint());
        Paint.setColor4f(Monet.get(monetRole::OnSurface));
        Surface->drawString(X.c_str(), (f32)XOff, Pos, Font, Paint);
      }

      XOff += m_optTabs[i].W;
    }

  }

  fun choice::doMouseMove(poit<f32> Pos, shiftStateSet State) -> void
  {
    i32 XOff = -10;

    i32 NHTabID = -1;

    for (u32 i = 0; i < m_optTabs.size(); i++)
    {
      XOff += 16;

      if (
        Pos.X > XOff -3 &&
        Pos.X < XOff +m_optTabs[i].W +3
      )
      {
        NHTabID = i;
        break;
      }

      XOff += m_optTabs[i].W;
    }


    if (NHTabID != m_hTabID)
    {
      m_hTabID = NHTabID;

      update(visDirtyDraw);
    }


    widget::doMouseMove(Pos, State);
  }

  fun choice::doClickEx(poit<f32> Pos) -> void
  {
    i32 XOff = -10;

    i32 NTabID = -1;

    for (u32 i = 0; i < m_optTabs.size(); i++)
    {
      XOff += 16;

      if (
        Pos.X > XOff -3 &&
        Pos.X < XOff +m_optTabs[i].W +3
      )
      {
        NTabID = i;
        break;
      }

      XOff += m_optTabs[i].W;
    }


    if (NTabID != -1 && NTabID != m_tabID)
      doChanged(NTabID);


    widget::doClickEx(Pos);
  }

  fun choice::calcAutoSize(renderContext& Context) -> void
  {
    SkFont Font(Context.font());
    Font.setSize(14);
    
    auto XOff = -3;
    for (const auto& X: m_tabs) {
      SkRect b;
      Font.measureText(X.c_str(), X.size(), SkTextEncoding::kUTF8, &b);
      XOff += b.width() + 16; 
    }
    
    setPreferedSize({XOff, 28});
  }

  #pragma endregion



  #pragma region grid

  fun grid::draw(renderContext& Context) -> void
  {
    auto Surface = Context.canvas();
    SkFont Font(Context.font());
    SkPaint Paint(Context.paint());
    
    Paint.setColor4f(Monet.get(monetRole::Surface));
    Paint.setStrokeWidth(1);
    
    for (int i = 1; i < m_rows.size()+1; i++)
      Surface->drawLine(0, (f32)i*m_rowHeight, (f32)size().W, (f32)i*m_rowHeight, Paint);
    
    Font.setSize(14);


    // Satırları Çiz
    int SX = 0;
    for (int C = 0; C < m_cols.size(); C++)
    {
      grid_column &Col = m_cols[C];
 
      Paint.setColor4f(Monet.get(monetRole::OnPrimary));
      Paint.setStyle(SkPaint::kFill_Style);

      Surface->drawString(Col.Title.c_str(), (f32)SX+5, (f32)0+7 + 10, Font, Paint); 


      // Items
      int SY = m_rowHeight;
      for (auto &R: m_rows)
      {
        if (C >= R.size())
          break;
        
        
        switch (Col.Type)
        {
          case grid_col_type::gctText:
          {
            Paint.setColor4f(Monet.get(monetRole::OnPrimary));
            Paint.setStyle(SkPaint::kFill_Style);
            Surface->drawString(R[C].c_str(), (f32)SX+5, (f32)SY+7 + 10, Font, Paint);
            break;
          }

          case grid_col_type::gctCheckbox:
          {
            // Draw Checkbox Inline
            Paint.setStrokeWidth(2);
            
            f32 CX = (f32)SX+4;
            f32 CY = (f32)SY+4;

            // Box
            Paint.setColor4f(Monet.get(monetRole::Surface));
            Paint.setStyle(SkPaint::kStroke_Style);
            for (auto &X: effects()) if (X->enabled()) X->modify(Paint);

            Surface->drawRoundRect(SkRect::MakeXYWH(CX+1, CY+1, 14, 14), 5, 5, Paint);

            if (R[C] == "1")
            {
              // Checked
              Paint.setColor4f(Monet.get(monetRole::Primary));
              Paint.setStyle(SkPaint::kFill_Style);
              for (auto &X: effects()) if (X->enabled()) X->modify(Paint);

              Surface->drawRoundRect(SkRect::MakeXYWH(CX+8-4, CY+8-4, 8, 8), 3, 3, Paint);
            }
            break;
          }

          default:;
        }

        SY += m_rowHeight;
      }
      SX += Col.Width;
    }

  }

  fun grid::doReset(renderContext& Ctx) -> void
  {
    m_lineColor.update();

    widget::doReset(Ctx);
  }

  fun grid::doClickEx(poit<f32> Pos) -> void
  {
    i32 nRow{-1}, nCol{-1};

    i32 Cac{};
    for (i32 i{}; i < m_cols.size(); i++)
      if ((Cac += m_cols[i].Width) > Pos.X)
      {
        nCol = i;
        break;
      }

    nRow = ((Pos.Y) /m_rowHeight) -1;

    if (nRow >= m_rows.size())
      return;

    doCellClick(nRow, nCol);    
  }

  fun grid::doCellClick(i32 Row, i32 Col) -> void
  {
    onCellClick(this, Row, Col);
  }

  fun grid::calcAutoSize(renderContext& Context) -> void
  {
    setPreferedSize({100,100});
  }

  #pragma endregion



  #pragma region popupMenu

  fun __popupMenu__propItems(const ds::value& Prop, function<expected<qev_seed,bool>(string_view)> FuncMap, popupMenu::popup_item* Item, string Scope) -> expected<bool, string>
  {
    if (!Prop.isArray()) return std::unexpected("was expected array, in "+Scope);

    for (int i{}; i < Prop.size(); i++) {
      auto X = Prop[i];

      if (X.isString()) {
        auto POP = new popupMenu::popup_item;
        POP->setParent(Item);
        Item->addItem(POP);
        
        POP->setName(X.w_string());
      }
      
      ef (X.isTuple()) {
        auto POP = new popupMenu::popup_item;
        Item->addItem(POP);
        POP->setParent(Item);
        
        if (auto Y = X[ds::etype::etStr]; !Y.isUnDef())
          POP->setName(Y.w_string());
        else
          return std::unexpected("was expected string in "+Scope+"["+to_string(i+1)+"]::string");


        if (auto Y = X[ds::etype::etArr]; !Y.isUnDef()) {
          if (auto Err = __popupMenu__propItems(Y, FuncMap, POP, Scope+"/"+POP->name()); !Err || !*Err)
            return Err;
        }


        if (auto Y = X[ds::etype::etStc]; !Y.isUnDef()) 
          for (int j{}; j < Y.size(); j++) {
            auto Q = Y.get_stc(j);

            if (Q.first == "onClick") {
              if (!Q.second.isString()) [[unlikely]]
              return std::unexpected("was expected string");
              
              auto Seed = FuncMap(Q.second.w_string());
              
              if (!Seed) [[unlikely]]
              return std::unexpected("no function in code");
              
              Seed->ToLoad(POP->onClick);
            }
            else
              return false;          
          }
      }

      else
        return std::unexpected("was expected string or tuple, in "+Scope+"["+to_string(i+1)+"]");
    }

    return true;
  }

  fun popupMenu::propItems(const ds::value& Prop, function<expected<qev_seed,bool>(string_view)> FuncMap) -> expected<bool, string>
  {
    return __popupMenu__propItems(Prop, FuncMap, &m_items, "/");
  }


  fun popupMenu::showPopup(poit<i32> Val) -> void
  {
    Val -= {PR.X1, PR.Y1};

    m_hSel = -1;

    m_nowItems = &m_items;


    if (auto Window = dynamic_cast<window*>(getRoot()))
    {
      setPoint(Val);
      Window->setOverlay(this);
    }
  }

  fun popupMenu::draw(renderContext& Context) -> void
  {
    auto Surface = Context.canvas();
    SkFont Font(Context.font());
    SkPaint Paint(Context.paint());

    Font.setSize(Context.sp(14));

    {
      auto rrect = SkRRect::MakeRectXY(SkRect::MakeXYWH(PR.X1*1.2,PR.Y1*1.2, (f32)width()-((PR.X1+PR.X2)*1.2), (f32)height()-((PR.Y1+PR.Y2)*1.2)), 13,13);

      SkPaint shadowPaint;
      shadowPaint.setColor(SK_ColorBLACK);
      shadowPaint.setAlphaf(0.5);
      shadowPaint.setAntiAlias(true);

      float blurSigma = 4.0;
      shadowPaint.setMaskFilter(SkMaskFilter::MakeBlur(kNormal_SkBlurStyle, blurSigma));

      Surface->drawRRect(rrect, shadowPaint);
    }


    Paint.setColor4f(Monet.get(monetRole::SurfaceContainerHigh));
    Paint.setStyle(SkPaint::kFill_Style);
    
    Surface->drawRoundRect(SkRect::MakeXYWH(PR.X1,PR.Y1, (f32)width()-PR.X1-PR.X2, (f32)height()-PR.Y1-PR.Y2), 13, 13, Paint);


    u32 s = PR.Y1;

    if (m_nowItems != &m_items)
    {
      SkRect TSize;
      string NameStr(m_nowItems->name());
      Font.measureText(NameStr.c_str(), NameStr.size(), SkTextEncoding::kUTF8, &TSize);

      
      Paint.setColor4f(Monet.get(monetRole::SurfaceContainer));
      Paint.setStyle(SkPaint::kFill_Style);
      SkVector radii[4] = {{13,13},{13,13}, {0,0},{0,0}};
      SkRRect RRect;
      RRect.setRectRadii(SkRect::MakeXYWH(PR.X1,PR.Y1, (f32)width()-PR.X1-PR.X2, (f32)PR.Y1+TSize.height()+16), radii);

      Surface->drawRRect(RRect, Paint);

      
      Paint.setColor4f(Monet.get(monetRole::OnSurfaceContainerHigh));
      Paint.setStyle(SkPaint::kStroke_Style);

      f32 x1 = PR.X1+ 17, y1 = (f32)s+11;
      f32 x2 = PR.X1+ 13, y2 = (f32)s+15;
      Surface->drawLine(x1, y1, x2, y2, Paint);
      
      f32 x3 = PR.X1 +17, y3 = (f32)s+19;
      Surface->drawLine(x2, y2, x3, y3, Paint);

      Paint.setStyle(SkPaint::kFill_Style);
      Surface->drawString(
        NameStr.c_str(), 
        ((f32)size().W -TSize.width())/2, 
        (s) +((30 -TSize.height())/2) - TSize.fTop, // Approximate baseline
        Font, 
        Paint
      );

      s += 30;
    }


    f32 itemHeight = Font.getSize()*1.8;
    f32 separatorHeight = 12.0;

    s += 2;

    for (u32 i = 0; i < m_nowItems->items().size(); i++)
    {
      auto item = m_nowItems->items()[i];
      string ItemNameStr(item->name());

      if (ItemNameStr == "-")
      {
        Paint.setColor4f(Monet.get(monetRole::SurfaceContainer));
        Paint.setStyle(SkPaint::kStroke_Style);
        Paint.setStrokeWidth(1.0f);
        
        f32 lineY = s + (separatorHeight / 2.0f);
        Surface->drawLine(PR.X1, lineY, (f32)width()-PR.X2, lineY, Paint);

        s += separatorHeight;
        continue;
      }

      SkRect TSize;
      Font.measureText(ItemNameStr.c_str(), ItemNameStr.size(), SkTextEncoding::kUTF8, &TSize);
      
      SkRect rowRect = SkRect::MakeXYWH(PR.X1+2, s, (f32)width()-PR.X1-PR.X2-4, itemHeight);

      if (i == m_hSel)
      {
        Paint.setColor4f(Monet.get(monetRole::PrimaryContainer));
        Paint.setStyle(SkPaint::kFill_Style);
        for (auto &X : effects()) if (X->enabled()) X->modify(Paint);
        
        Surface->drawRRect(SkRRect::MakeRectXY(rowRect.makeInset(2, 2), 13, 13), Paint);
      }

      if (i == m_hSel)
        Paint = SkPaint(Monet.get(monetRole::OnPrimaryContainer));
      else
        Paint = SkPaint(Monet.get(monetRole::OnSurfaceContainerHigh));
      
      Paint.setAntiAlias(true);


      f32 textY = s + (itemHeight /2.0) -(TSize.fTop +TSize.fBottom) /2.0;

      Surface->drawString(ItemNameStr.c_str(), rowRect.fLeft +8, textY, Font, Paint);


      if (!item->items().empty())
      {
        Paint.setStyle(SkPaint::kStroke_Style);
        Paint.setStrokeWidth(1);
        
        f32 arrowX = (f32)width() -PR.X2 -13.0;
        f32 arrowY = s + (itemHeight / 2.0);
        f32 size = 4.0;

        Surface->drawLine({arrowX -size, arrowY -size}, {arrowX, arrowY}, Paint);
        Surface->drawLine({arrowX, arrowY}, {arrowX -size, arrowY +size}, Paint);
      }

      s += itemHeight; // Bir sonraki öğeye geç
    }
    s += PR.Y2 -2;


    if (s+4 != height())
    {
      pair<u32, popupMenu*> *Data = new pair<u32, popupMenu*>;

      Data->first = s+4;
      Data->second = this;


      CurrentApp->pushTask(
        [](u0 Data)
        {
          pair<u32, popupMenu*> *Pair = (pair<u32, popupMenu*>*)Data;

          Pair->second->setSize({Pair->second->width(), (i32)Pair->first});

          CurrentApp->pushMessage(Pair->second->getRoot(), messages::vmPaint);

          delete Pair;
        },
        (u0)Data
      );
    }
  }

  fun popupMenu::doMouseUp(poit<f32> Pos, shiftStateSet Button, shiftStateSet State) -> void
  {
    Pos -= {(f32)PR.X1, (f32)PR.Y1};

    const i32 k = 14*1.8;
    u32 s = 3;
    i32 NHSel = 0;

    if (Pos.X < 0 || Pos.Y < 0 || Pos.X > width()-PR.X1-PR.X2 || Pos.Y > height()-PR.Y1-PR.Y1)
    {
      NHSel = -1;
      goto _l_Sec;
    }
    
    if (m_nowItems != &m_items)
    {
      s += 30;
      
      if (Pos.Y <= s)
      {
        NHSel = -2;
        goto _l_Sec;
      }
    }

    for (u32 i = 0; i < m_nowItems->items().size(); i++)
    {
      if (m_nowItems->items()[i]->name() == "-")
        s += 12;
      else
        s += k;

      if (s >= Pos.Y)
        break;

      NHSel += 1;
    }


    _l_Sec:
    if (NHSel == -2)
    {
      if (m_nowItems->items()[0]->parent() == Nil)
        return;

      m_nowItems = m_nowItems->parent();

      doMouseMove(Pos, 0);

      update(visDirtyDraw);
      return;
    }


    if (Pos.X < 0 || Pos.X > width())
      goto _l_Close;

    if (m_nowItems->items().size() <= NHSel)
      return;

    if (m_nowItems->items()[NHSel]->name() == "-")
      return;



    if (m_nowItems->items()[NHSel]->items().empty())
    {
      if (auto win = dynamic_cast<window*>(getRoot()))
      {
        win->setOverlay(Nil);
      }
      
      m_nowItems->items()[NHSel]->onClick(this);
      
      update();
    }
    else
    {
      m_nowItems->items()[NHSel]->setParent(m_nowItems);
      m_nowItems = m_nowItems->items()[NHSel];

      doMouseMove(Pos, 0);

      update(visDirtyDraw);
    }


    return;

    _l_Close:
    if (auto win = dynamic_cast<window*>(getRoot()))
    {
      win->setOverlay(Nil);
      update();
    }
  }

  fun popupMenu::doMouseMove(poit<f32> Pos, shiftStateSet State) -> void
  {
    Pos -= {(f32)PR.X1, (f32)PR.Y1};


    const i32 k = 14*1.8;
    u32 s = 3;
    i32 NHSel = 0;

    if (Pos.X < 0 || Pos.Y < 0 || Pos.X > width()-PR.X1-PR.X2 || Pos.Y > height()-PR.Y1-PR.Y1)
    {
      NHSel = -1;
      goto _l_Sec;
    }

    if (m_nowItems != &m_items)
    {
      s += 30;
      
      if (Pos.Y <= s)
      {
        NHSel = -2;
        goto _l_Sec;
      }
    }

    for (u32 i = 0; i < m_nowItems->items().size(); i++)
    {
      if (m_nowItems->items()[i]->name() == "-")
        s += 12;
      else
        s += k;

      if (s >= Pos.Y)
        break;

      NHSel += 1;
    }
    

    _l_Sec:
    if (NHSel != m_hSel)
    {
      m_hSel = NHSel;

      update(visDirtyDraw);
    }

    contextMenu::doMouseMove(Pos, State);
  }

  fun popupMenu::handlerStateChanged(visual::stateFlagSet State) -> void
  {
    contextMenu::handlerStateChanged(State);

    if ((State & stateFlags::sfHover) == 0)
      m_hSel = -1;


    update(visDirtyDraw);
  }

  #pragma endregion



  #pragma region chip

  fun chip::draw(renderContext& Context) -> void
  {
    auto Surface = Context.canvas();
    SkPaint Paint(Context.paint());
    
    Paint.setStrokeWidth(2);
    Paint.setColor4f(m_color);
    for (auto &X: effects()) if (X->enabled()) X->modify(Paint);

    Paint.setStyle(SkPaint::kStroke_Style);
    
    f32 Radius = m_borderRadius;
    if (Radius < 0)
       Radius = (size().W > height() ? width():size().H);
       
    Surface->drawRoundRect(SkRect::MakeLTRB(1,1, (f32)width()-1, (f32)height()-1), Radius, Radius, Paint);
  }

  fun chip::doReset(renderContext& Ctx) -> void
  {
    m_color.update();

    view::doReset(Ctx);
  }

  #pragma endregion


  #pragma region card

  fun card::draw(renderContext& Context) -> void
  {
    auto Surface = Context.canvas();
    SkPaint Paint(Context.paint());

    f32 Radius = m_borderRadius;
    if (Radius < 0)
       Radius = (size().W > height() ? width():size().H);
    
    // Clip
    SkRRect RRect;
    RRect.setRectXY(SkRect::MakeXYWH(0,0, (f32)width(), (f32)height()), Radius, Radius);

    Paint.setColor4f(m_color);
    Paint.setStyle(SkPaint::kFill_Style);
    for (auto &X: effects()) if (X->enabled()) X->modify(Paint);
    
    Surface->drawRRect(RRect, Paint);
  }

  fun card::doReset(renderContext& Ctx) -> void
  {
    m_color.update();

    view::doReset(Ctx);
  }

  #pragma endregion

}
