/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


#pragma once

#include "ds/ds.hh"
#include "qcl/widget.hh"

using namespace qcl;



namespace qstd
{

  struct qcl_object interval: qcl::widget
  {
    public:
      inline interval() { setSize({140, 28}); }
      

    private:
      u32 m_max{100}, m_pos1{30}, m_pos2{70};


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

      
      inline fun pos1() const noexcept -> u32 { return m_pos1; }
      inline fun setPos1(u32 val) -> void { if (m_pos1 == val) return; m_pos1 = val; update(visDirtyDraw); }
      inline fun propPos1(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isInt())
          return std::unexpected("was expected int");
      
        setPos1(Prop.w_int());
        return true;
      }
      qcl_prop(TYPE u32, NAME pos1, READ pos1, WRITE setPos1, PROP propPos1);


      inline fun pos2() const noexcept -> u32 { return m_pos2; }
      inline fun setPos2(u32 val) -> void { if (m_pos2 == val) return; m_pos2 = val; update(visDirtyDraw); }
      inline fun propPos2(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isInt())
          return std::unexpected("was expected int");
      
        setPos2(Prop.w_int());
        return true;
      }
      qcl_prop(TYPE u32, NAME pos2, READ pos2, WRITE setPos2, PROP propPos2);


    public:
      qcl_func qev<void, u32 /* Pos1 */, u32 /* Pos2 */> onChanged;


    public:
      fun draw(renderContext&) -> void override;

      fun handlerStateChanged(qcl::visual::stateFlagSet State) -> void override;
      fun doMouseDown(poit<f32> Pos, shiftStateSet Button, shiftStateSet State) -> void override;
      fun doMouseMove(poit<f32> Pos, shiftStateSet State) -> void override;
      virtual fun doChanged(u32 Pos1, u32 Pos2) -> void;

    public:
      fun getProp(string_view) -> expected<std::any, bool> override;
      fun setProp(string_view, std::any) -> expected<void, bool> override;
      fun loadProp(string_view, const ds::value&, function<expected<qev_seed,bool>(string_view)>) -> expected<bool, string> override;
  };

}
