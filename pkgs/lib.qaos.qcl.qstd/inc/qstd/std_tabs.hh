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
      fun getProp(string_view) -> expected<std::any, bool> override;
      fun setProp(string_view, std::any) -> expected<void, bool> override;
      fun loadProp(string_view, const ds::value&, function<expected<qev_seed,bool>(string_view)>) -> expected<bool, string> override;
  };

}
