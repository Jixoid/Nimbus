/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


#include <expected>
#include <random>
#include <string_view>
#include "ds/ds.hh"
#include "include/core/SkBlendMode.h"
#include "include/core/SkImage.h"
#include "include/core/SkBitmap.h"
#include "include/core/SkColor.h"
#include "include/core/SkPaint.h"
#include "include/core/SkColorFilter.h"
#include "include/core/SkShader.h"
#include "include/core/SkSamplingOptions.h"
#include "include/core/SkTileMode.h"
#include "qcl/dynamicLoad.hh"
#include "qcl/effect.hh"
#include "qstd/effects.hh"

using namespace std;
using namespace qcl;



namespace qstd
{
  #define regE(X) {#X, {[]() -> effect* { return new X(); }, [](effect* val) -> bool { return dynamic_cast<X*>(val); }}}

  const dyn::effs _effs = {
    regE(noise),
    regE(grayscale),
  };

  #undef regE

  fun effs() -> dyn::effs { return _effs; }




  #pragma region noise

  fun noise::reset() -> void
  {
    SkBitmap Bitmap;
    Bitmap.allocN32Pixels(N_SIZE, N_SIZE);


    // RNG
    random_device rd;
    mt19937 mt(rd());
    uniform_int_distribution<u8> dist(0, 255);

    
    // Generate
    u32* Pixels = (u32*)Bitmap.getPixels();

    for (int i = 0; i < N_SIZE * N_SIZE; i++)
    {
      u8 val = static_cast<u8>(dist(mt));

      *(Pixels++) = SkColorSetARGB(255, val,val,val);
    }

    m_noiseShader = Bitmap.makeShader(SkTileMode::kRepeat, SkTileMode::kRepeat, SkSamplingOptions(SkFilterMode::kNearest));
  }

  fun noise::modify(SkPaint& Paint) -> void
  {
    if (!enabled() || !m_noiseShader) return;

    sk_sp<SkShader> currentShader = Paint.refShader();

    if (!currentShader)
      currentShader = SkShaders::Color(Paint.getColor4f(), nil); 
    

    auto noiseWithOpacity = m_noiseShader->makeWithColorFilter(
      SkColorFilters::Blend(color::white() *vnoise(), nil, SkBlendMode::kDstIn)
    );
    
    
    
    auto composedShader = SkShaders::Blend(
      SkBlendMode::kOverlay, // Karışım modu
      currentShader,
      noiseWithOpacity
    );

    // 3. Yeni birleşik shader'ı Paint'e ata
    Paint.setShader(composedShader);
  }

  fun noise::loadProp(string_view Name, const ds::value& Prop) -> std::expected<bool, std::string>
  {
    if (auto Err = effect::loadProp(Name, Prop); !Err || *Err) return Err;


    if (Name == "noise")
    {
      if (!Prop.isInt()) return std::unexpected("was expected int");

      this->setVNoise(Prop.w_int());
      return true;
    }

    else
      return std::unexpected("");
  }

  #pragma endregion


  #pragma region gray

  fun grayscale::modify(SkPaint& Paint) -> void
  {
    if (!enabled()) return;

    const float matrix[20] = {
      0.2126f, 0.7152f, 0.0722f, 0.0f, 0.0f, // R' = 0.21R + 0.71G + 0.07B
      0.2126f, 0.7152f, 0.0722f, 0.0f, 0.0f, // G' = aynısı
      0.2126f, 0.7152f, 0.0722f, 0.0f, 0.0f, // B' = aynısı
      0.0f,    0.0f,    0.0f,    1.0f, 0.0f  // A' = Alpha (Değişmez)
    };
    
    sk_sp<SkColorFilter> grayFilter = SkColorFilters::Matrix(matrix);

    sk_sp<SkColorFilter> currentFilter = Paint.refColorFilter();


    if (currentFilter)
      Paint.setColorFilter(SkColorFilters::Compose(grayFilter, currentFilter));
    
    else
      Paint.setColorFilter(grayFilter);
  }

  #pragma endregion

}
