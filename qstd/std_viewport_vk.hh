/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
*/

#pragma once

#include "qcl/ds/ds.hh"
#include "qcl/widget.hh"
#include "include/core/SkSurface.h"
#include "include/core/SkRefCnt.h"

using namespace qcl;

namespace qstd
{
  
  struct qcl_object viewPortVK: qcl::widget
  {
    private:
      u64 m_imageHandle{0};
      i32 m_textureWidth{0};
      i32 m_textureHeight{0};
      u32 m_textureFormat{37}; // VK_FORMAT_R8G8B8A8_UNORM
      u32 m_imageLayout{0}; // VK_IMAGE_LAYOUT_UNDEFINED
      u32 m_imageUsageFlags{4}; // VK_IMAGE_USAGE_SAMPLED_BIT
      u64 m_vkInfoPtr{0};

    public:
      inline fun imageHandle() const -> u64 { return m_imageHandle; }
      inline fun setImageHandle(u64 val) -> void { m_imageHandle = val; update(visDirtyDraw); }
      inline fun propImageHandle(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isInt()) return std::unexpected("was expected int");
        setImageHandle(static_cast<u64>(Prop.w_int()));
        return true;
      }
      qcl_prop(TYPE u64, NAME imageHandle, READ imageHandle, WRITE setImageHandle, PROP propImageHandle);

      inline fun textureWidth() const -> i32 { return m_textureWidth; }
      inline fun setTextureWidth(i32 val) -> void { m_textureWidth = val; update(visDirtyDraw); }
      inline fun propTextureWidth(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isInt()) return std::unexpected("was expected int");
        setTextureWidth(Prop.w_int());
        return true;
      }
      qcl_prop(TYPE i32, NAME textureWidth, READ textureWidth, WRITE setTextureWidth, PROP propTextureWidth);

      inline fun textureHeight() const -> i32 { return m_textureHeight; }
      inline fun setTextureHeight(i32 val) -> void { m_textureHeight = val; update(visDirtyDraw); }
      inline fun propTextureHeight(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isInt()) return std::unexpected("was expected int");
        setTextureHeight(Prop.w_int());
        return true;
      }
      qcl_prop(TYPE i32, NAME textureHeight, READ textureHeight, WRITE setTextureHeight, PROP propTextureHeight);

      inline fun textureFormat() const -> u32 { return m_textureFormat; }
      inline fun setTextureFormat(u32 val) -> void { m_textureFormat = val; update(visDirtyDraw); }
      inline fun propTextureFormat(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isInt()) return std::unexpected("was expected int");
        setTextureFormat(Prop.w_int());
        return true;
      }
      qcl_prop(TYPE u32, NAME textureFormat, READ textureFormat, WRITE setTextureFormat, PROP propTextureFormat);

      inline fun imageLayout() const -> u32 { return m_imageLayout; }
      inline fun setImageLayout(u32 val) -> void { m_imageLayout = val; update(visDirtyDraw); }
      inline fun propImageLayout(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isInt()) return std::unexpected("was expected int");
        setImageLayout(Prop.w_int());
        return true;
      }
      qcl_prop(TYPE u32, NAME imageLayout, READ imageLayout, WRITE setImageLayout, PROP propImageLayout);

      inline fun imageUsageFlags() const -> u32 { return m_imageUsageFlags; }
      inline fun setImageUsageFlags(u32 val) -> void { m_imageUsageFlags = val; update(visDirtyDraw); }
      inline fun propImageUsageFlags(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isInt()) return std::unexpected("was expected int");
        setImageUsageFlags(Prop.w_int());
        return true;
      }
      qcl_prop(TYPE u32, NAME imageUsageFlags, READ imageUsageFlags, WRITE setImageUsageFlags, PROP propImageUsageFlags);

      inline fun vkInfoPtr() const -> u64 { return m_vkInfoPtr; }
      inline fun setVkInfoPtr(u64 val) -> void { m_vkInfoPtr = val; update(visDirtyDraw); }
      inline fun propVkInfoPtr(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isInt()) return std::unexpected("was expected int");
        setVkInfoPtr(Prop.w_int());
        return true;
      }
      qcl_prop(TYPE u64, NAME vkInfoPtr, READ vkInfoPtr, WRITE setVkInfoPtr, PROP propVkInfoPtr);

    public:
      fun draw(renderContext&) -> void override;

    public:
      fun getProp(string_view) -> expected<std::any, bool> override;
      fun setProp(string_view, std::any) -> expected<void, bool> override;
      fun loadProp(string_view, const ds::value&, function<expected<qev_seed,bool>(string_view)>) -> expected<bool, string> override;
  };

}
