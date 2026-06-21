/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
*/

#include "qstd/std_viewport_vk.hh"
#include "qcl/basis.hh"
#include <iostream>
#include "include/core/SkCanvas.h"
#include "include/core/SkImage.h"
#include "include/core/SkColorSpace.h"
#include "include/gpu/ganesh/SkImageGanesh.h"
#include "include/gpu/ganesh/GrBackendSurface.h"
#include "include/gpu/ganesh/vk/GrVkBackendSurface.h"
#include "include/gpu/ganesh/vk/GrVkTypes.h"

namespace qstd
{

  fun viewPortVK::draw(renderContext& Context) -> void
  {
    if (m_imageHandle == 0 || m_textureWidth <= 0 || m_textureHeight <= 0)
      return;

    auto Canvas = Context.canvas();
    auto recordingContext = Canvas->recordingContext();
    if (!recordingContext) return;

    GrVkImageInfo vkInfo{};
    if (m_vkInfoPtr != 0) {
      vkInfo = *reinterpret_cast<GrVkImageInfo*>(m_vkInfoPtr);
    } else {
      vkInfo.fImage = reinterpret_cast<VkImage>(m_imageHandle);
      vkInfo.fImageLayout = static_cast<VkImageLayout>(m_imageLayout);
      vkInfo.fFormat = static_cast<VkFormat>(m_textureFormat);
      vkInfo.fLevelCount = 1;
      vkInfo.fImageUsageFlags = m_imageUsageFlags;
    }

    auto backendTexture = GrBackendTextures::MakeVk(m_textureWidth, m_textureHeight, vkInfo);
    
    auto image = SkImages::BorrowTextureFrom(recordingContext, backendTexture, kTopLeft_GrSurfaceOrigin, kRGBA_8888_SkColorType, kPremul_SkAlphaType, nullptr);
    if (!image) {
      std::cout << "viewPortVK::draw -> SkImages::BorrowTextureFrom FAILED! handle=" 
                << m_imageHandle << ", format=" << m_textureFormat 
                << ", layout=" << m_imageLayout << std::endl;
      return;
    }

    Canvas->save();
    
    SkRect src = SkRect::MakeIWH(m_textureWidth, m_textureHeight);
    SkRect dst = SkRect::MakeXYWH(0, 0, Context.dp(width()), Context.dp(height()));
    
    Canvas->drawImageRect(image, src, dst, SkSamplingOptions(SkFilterMode::kLinear), nullptr, SkCanvas::kFast_SrcRectConstraint);

    Canvas->restore();
  }
  
}
