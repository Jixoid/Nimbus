/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


#pragma once

#include "qcl/ds/ds.hh"
#include "qcl/popup.hh"

using namespace qcl;



namespace qstd
{

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

            return nil;
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
      fun getProp(string_view) -> expected<std::any, bool> override;
      fun setProp(string_view, std::any) -> expected<void, bool> override;
      fun loadProp(string_view, const ds::value&, function<expected<qev_seed,bool>(string_view)>) -> expected<bool, string> override;
  };

}
