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

  struct qcl_object multi_progbar: qcl::widget
  {
    public:
      inline multi_progbar() { setSize({140, 24}); }


    private:
      vector<pair<u32, dcolor>> m_values{};


    public:
      inline fun getVals() const -> vector<pair<u32, dcolor>> { return m_values; }
      inline fun setVals(vector<pair<u32, dcolor>> val) -> void { m_values = val; update(visDirtyAutoSize | visDirtyDraw); }
      inline fun propVals(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isArray()) return std::unexpected("was expected array");

        vector<pair<u32, dcolor>> Nat;

        for (int i{}; i < Prop.size(); i++) {
          if (auto X = Prop[i]; X.isInt())
            Nat.push_back({X.w_int(), dcolor(this, monetRole::PrimaryContainer)});
          ef (auto X = Prop[i]; X.isTuple()) {
            pair<u32, dcolor> A = {0, dcolor(this, monetRole::PrimaryContainer)};

            if (auto C = X[ds::etype::etI64]; !X.isUnDef())
              A.first = C.w_int();
            else
              return std::unexpected("was expected tuple<int, string>, in /["+to_string(i+1)+"]");

            if (auto C = X[ds::etype::etStr]; !X.isUnDef())
              A.second.update(C.w_string());
            else
              return std::unexpected("was expected tuple<int, string>, in /["+to_string(i+1)+"]");

            Nat.push_back(A);
          }
          else
            return std::unexpected("was expected int or tuple<int, string>, in /["+to_string(i+1)+"]");
        }

        setVals(Nat);
        return true;
      }
      qcl_prop(TYPE vector<pair<u32, dcolor>>, NAME vals, READ getVals, WRITE setVals, PROP propVals);
      

    public:
      fun draw(renderContext&) -> void override;

    public:
      fun getProp(string_view) -> expected<std::any, bool> override;
      fun setProp(string_view, std::any) -> expected<void, bool> override;
      fun loadProp(string_view, const ds::value&, function<expected<qev_seed,bool>(string_view)>) -> expected<bool, string> override;
  };

}
