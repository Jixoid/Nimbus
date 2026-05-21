/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


#include "qcl/basis.hh"
#include "qcl/vfs/vfs.hh"
#include <iostream>
#include "include/core/SkColorFilter.h"
#include "include/core/SkSurface.h"
#include "include/core/SkData.h"
#include "include/core/SkStream.h"
#include "include/core/SkImage.h"
#include "modules/svg/include/SkSVGDOM.h"
#include "qcl/types.hh"
#include "qstd/standard.hh"



namespace qstd
{

  fun icon::draw(renderContext& Context) -> void
  {
    if (m_icon == nil)
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
    vector<char> buffer((istreambuf_iterator<char>(*svgStream)), istreambuf_iterator<char>());
    
    if (buffer.empty()) {
      cerr << "HATA: SVG akışı boş!" << endl;
      return;
    }

    auto data = SkData::MakeWithCopy(buffer.data(), buffer.size());
    auto skStream = SkMemoryStream::Make(data);

    auto dom = SkSVGDOM::Builder().setFontManager(fontMgr).make(*skStream);
    if (!dom) {
      cerr << "HATA: SVG parse edilemedi!" << endl;
      return;
    }

    
    SkSize svgSize = dom->containerSize();
    if (svgSize.isEmpty()) {
      svgSize = SkSize::Make(100, 100);
      dom->setContainerSize(svgSize);
    }

    targetCanvas->save();
    targetCanvas->translate(x, y);

    
    if (targetWidth > 0 && svgSize.width() > 0) {
      float scale = targetWidth / svgSize.width();
      targetCanvas->scale(scale, scale);
    }

    dom->render(targetCanvas);

    targetCanvas->restore();
  }


  fun icon::doPaint_prepare(renderContext& Context) -> void
  {
    if (!m_reRender)
      widget::doPaint_prepare(Context);


    if (m_icon) m_icon = nil;

    if (!m_path.empty())
    {
      auto Stream = vfs::resolve_ro(m_path);
    
      auto ImgInfo = SkImageInfo::MakeN32Premul(Context.dp(width()), Context.dp(height()));
      m_icon = SkSurfaces::Raster(ImgInfo);

      auto Canv = m_icon->getCanvas();

      PasteSvgOnCanvas(Canv, Context.fontMgr(), Stream.get(), 0,0, Context.dp(width()));
    }

    m_reRender = false;
    

    widget::doPaint_prepare(Context);
  }
  
}
