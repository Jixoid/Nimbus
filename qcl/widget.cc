/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


#include "qcl/basis.hh"
#include "qcl/ds/ds.hh"
#include "qcl/visual.hh"
#include "qcl/widget.hh"
#include "qcl/popup.hh"
#include "qcl/view.hh"
#include "qcl/types.hh"

#define el else
#define ef else if

using namespace std;



namespace qcl
{
  fun widgetAnchor::update() -> void
  {
    if (m_owner->parent())
      m_owner->parent()->addFlag(visual::dirtyFlags::vieDirtyTiling);
  }
  
  fun widgetMargins::update() -> void
  {
    if (m_owner->parent())
      m_owner->parent()->addFlag(visual::dirtyFlags::vieDirtyTiling);
  }


  fun __findInScope(widget* Self, string Name) -> widget*
  {
    if (Self->parent() == nil)
      return nil;


    if (Self->parent()->name() == Name)
      return Self->parent();

    
    for (auto &X: Self->parent()->widgets())
      if (X != Self && X->name() == Name)
        return X;

    return nil;
  }


  fun widget::propAnchors(const ds::value& Prop) -> expected<bool, string>
  {
    if (!Prop.isStruct()) return std::unexpected("was expected struct");


    auto Handler = [this](ds::value& Tup, widgetAnchor& Anchor) -> pair<bool, string>
    {
      if (Tup.isNull()) {
        Anchor.setActive(false);
        Anchor.setControl(nil);

        return {true, ""};
      }

      ef (Tup.isBool()) {
        Anchor.setActive(Tup.w_bool());
        
        return {true, ""};
      }


      if (!Tup.isTuple())
        return {false, " is not a tuple, null or bool"};

      
      // Active
      Anchor.setActive(true);

      // Control
      if (auto Buf = Tup[ds::etype::etRef]; !Buf.isUnDef()) {
        auto Cac = Buf.w_ref();

        if (Cac == "^")
          Anchor.setControl(parent());
        el
          Anchor.setControl(__findInScope(this, Cac));

        if (!Anchor.control())
          return {false, "/control is not finded"};
      }

      // Side
      if (auto Buf = Tup[ds::etype::etStr]; !Buf.isUnDef()) {
        auto Cac = Buf.w_string();

        if (Cac == "<-") Anchor.setSide(widgetAnchorSide::wasBeg);
        ef (Cac == "->") Anchor.setSide(widgetAnchorSide::wasEnd);
        ef (Cac == "><") Anchor.setSide(widgetAnchorSide::wasCen);

        else
          return {false, "/side (use '<-' | '->' | '><')"};
      }

      return {true, ""};
    };


    if (auto Buf = Prop["left"]; !Buf.isUnDef())
      if (auto Err = Handler(Buf, anchors().left()); !Err.first)
        return std::unexpected(Err.second +" left" +Err.second);

    if (auto Buf = Prop["top"]; !Buf.isUnDef())
      if (auto Err = Handler(Buf, anchors().top()); !Err.first)
        return std::unexpected(Err.second +" top" +Err.second);

    if (auto Buf = Prop["righ"]; !Buf.isUnDef())
      if (auto Err = Handler(Buf, anchors().right()); !Err.first)
        return std::unexpected(Err.second +" righ" +Err.second);

    if (auto Buf = Prop["bot"]; !Buf.isUnDef())
      if (auto Err = Handler(Buf, anchors().bottom()); !Err.first)
        return std::unexpected(Err.second +" bot" +Err.second);

    return true;
  }

  fun widget::propMargins(const ds::value& Prop) -> expected<bool, string>
  {
    if (!Prop.isStruct()) return std::unexpected("was expected struct");


    for (int i{}; i < Prop.size(); i++) {
      const auto X = Prop.get_stc(i);

      if (X.first == "left") {
        if (!X.second.isInt()) return std::unexpected("was expected int");
        margins().setLeft(X.second.w_int());
      }
      ef (X.first == "righ") {
        if (!X.second.isInt()) return std::unexpected("was expected int");
        margins().setRight(X.second.w_int());
      }
      ef (X.first == "top") {
        if (!X.second.isInt()) return std::unexpected("was expected int");
        margins().setTop(X.second.w_int());
      }
      ef (X.first == "bot") {
        if (!X.second.isInt()) return std::unexpected("was expected int");
        margins().setBottom(X.second.w_int());
      }
      else
        return std::unexpected("unexpected property: "+X.first);
    }

    return true;
  }


  fun widget::doMouseDown(poit<f32> Pos, shiftStateSet Button, shiftStateSet State) -> void
  {
    visual::doMouseDown(Pos, Button, State);

    if (m_contextMenu && State == 0 && Button == shiftStates::ssRight)
      m_contextMenu->showPopup(Pos+getLocalPos(getRoot()));
  }

}
