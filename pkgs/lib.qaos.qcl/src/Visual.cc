/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/



#include "qcl/Application.hh"
#include "qcl/Scrolling.hh"
#include <algorithm>
#define el else
#define ef else if

#include "Basis.h"

#include "DS.hh"

#include "qcl/Visual.hh"
#include "qcl/Window.hh"
#include "qcl/Effect.hh"
#include "qcl/DynamicLoad.hh"
#include "qcl/View.hh"
#include "qcl/Types.hh"

using namespace std;



u64 GetTickCount()
{
  struct timeval tv;
  gettimeofday(&tv, nullptr);
  
  return static_cast<u64>(tv.tv_sec) *1000 +tv.tv_usec /1000;
}


namespace qcl
{

  fun visualPaddings::update() -> void
  {
    if (m_owner->parent())
      m_owner->parent()->addFlag(visual::dirtyFlags::vieDirtyTiling);
  }

  fun visual::propPaddings(const ds::value& Prop) -> expected<bool, string>
  {
    if (!Prop.isStruct()) return std::unexpected("was expected struct");


    for (int i{}; i < Prop.size(); i++) {
      const auto X = Prop.get_stc(i);

      if (X.first == "left") {
        if (!X.second.isInt()) return std::unexpected("was expected int");
        paddings().setLeft(X.second.w_int());
      }
      ef (X.first == "righ") {
        if (!X.second.isInt()) return std::unexpected("was expected int");
        paddings().setRight(X.second.w_int());
      }
      ef (X.first == "top") {
        if (!X.second.isInt()) return std::unexpected("was expected int");
        paddings().setTop(X.second.w_int());
      }
      ef (X.first == "bot") {
        if (!X.second.isInt()) return std::unexpected("was expected int");
        paddings().setBottom(X.second.w_int());
      }
      else
        return std::unexpected("unexpected property: "+X.first);
    }

    return true;
  }


  fun visual::relEffect(effect* Val) -> effect*
  {
    auto it = std::find(m_effects.begin(), m_effects.end(), Val);
    auto Ret = *it;
    m_effects.erase(it);

    return Ret;
  }

  fun visual::calcAutoSize(renderContext&) -> void { setPreferedSize({20,20}); }

  fun visual::dyeToRoot(dirtyFlagSet Flag) -> void
  {
    visual* This{this};
    This->addFlag(Flag);

    while (This->parent()) {
      This = This->parent();
      This->addFlag(Flag);
    }
  }


  fun visual::addFlag(dirtyFlagSet Flag) noexcept -> void
  {
    // 1. Kendi içsel (local) bayrak bağımlılıklarını çöz
    if (Flag & visDirtyResize)
      Flag |= (visDirtyRebound | vieDirtyTiling);
      
    // YENİ EKLENEN: Bir şeyin boyutu otomatik hesaplanacaksa, kesinlikle yeniden dizilmelidir (Tiling)
    if (Flag & visDirtyAutoSize)
      Flag |= vieDirtyTiling; 

    if (Flag & vieDirtyTiling)
      Flag |= visDirtyDraw;

    // Önce kendi bayrağımızı atayalım
    m_dirtyFlags |= Flag;

    // 2. Parent'lara durumu bildir (Tek Döngü - Single Tree Walk)
    visual* parentNode = this->parent();
    
    // Yukarıya (parent'lara) hangi bayrakları bulaştıracağız?
    dirtyFlagSet propagateFlags = 0;
    
    if (Flag & visDirtyDraw)
      propagateFlags |= visDirtyDraw;
      
    // Eğer child'da herhangi bir layout/boyut değişimi varsa, parent YENİDEN DİZİLMEK (Tiling) zorundadır.
    if (Flag & (visDirtyResize | visDirtyRebound | visDirtyAutoSize | vieDirtyTiling)) {
      propagateFlags |= (vieDirtyTiling | visDirtyDraw);
    }

    // Yukarı taşınacak bir şey yoksa boşuna döngüye girme
    if (propagateFlags == 0) return;

    while (parentNode) {
      // Parent'a bulaştırılması gereken temel bayrakları ekle
      parentNode->m_dirtyFlags |= propagateFlags;

      // Eğer layout kirliliği yukarı taşınıyorsa ve bu parent kendi boyutunu child'lara göre ayarlıyorsa:
      if ((propagateFlags & vieDirtyTiling) && (parentNode->autoSizeVert() || parentNode->autoSizeHorz())) {
        parentNode->m_dirtyFlags |= visDirtyAutoSize;
      }

      parentNode = parentNode->parent();
    }
  }

  fun visual::remFlag(dirtyFlagSet Flag) noexcept -> void
  {
    m_dirtyFlags &= ~Flag;
  }



  fun visual::update(dirtyFlagSet Flag) -> void
  {
    addFlag(Flag);
    if (auto W = dynamic_cast<window*>(getRoot()); W)
      CurrentApp->pushMessage(W, visual::vmPaint);
  }


  fun visual::getLocalPos(view *Target) const noexcept -> poit<i32>
  {
    poit<i32> Ret{};
    auto This = (widget*)this;

    while (true)
    {
      Ret += This->point();

      if (auto C = dynamic_cast<scrolling*>(This->parent()))
        Ret += C->clientPos();


      if (!This->parent())
        return {0,0};

      ef (This->parent() == Target)
        return Ret;

      el
        This = This->parent();
    }
    
  }
  

  fun visual::propEffects(const ds::value& Prop) -> expected<bool, string>
  {
    if (!Prop.isArray()) return std::unexpected("was expected array");
  
    vector<effect*> Nat;

    for (u32 i{}; i < Prop.size(); i++)
    {
      auto X = Prop[i];

      if (!X.isTuple()) return std::unexpected("was expected tuple");

      
      pair<string, string> Typ;
      if (auto C = X[ds::etype::etTyp]; !C.isUnDef())
        Typ = C.w_type();
      else
        return std::unexpected("was expected type in tuple");

      
      auto itG = qcl::dyn::EffList.find(Typ.first);
      if (itG == dyn::EffList.end())
        return std::unexpected("there type not found: \""+Typ.first+"::"+Typ.second+"\"");

      auto itO = itG->second.find(Typ.second);
      if (itO == itG->second.end())
        return std::unexpected("there type not found: \""+Typ.first+"::"+Typ.second+"\"");

      Nat.push_back(itO->second.first());


      if (auto C = X[ds::etype::etStc]; !C.isUnDef())
        for (u32 j = 0; j < C.size(); j++)
        {
          auto SX = C.get_stc(j);
          
          if (auto Err = Nat.back()->loadProp(SX.first, SX.second); !Err || !*Err) return Err;
        }
      else
        return std::unexpected("was expected struct in tuple");
    }
  

    for (auto &X: m_effects)
      delete X;
    m_effects.clear();

    for (auto &X: Nat)
      addEffect(X);
    
    return true;
  }


  fun visual::handlerPaint(renderContext& Val) -> void
  {
    doPaint_prepare(Val);
    doPaint(Val);
  }

  fun visual::handlerResize() -> void
  {
    addFlag(visDirtyResize);
  }

  fun visual::handlerStateChanged(stateFlagSet State) -> void
  {
    m_stateFlags = State;

    doStateChanged(State);
  }



  fun visual::doReset(renderContext&) -> void
  {
    update();
  }

  fun visual::doPaint(renderContext& Surface) -> void
  {
    draw_before(Surface);
    if (EPaint) EPaint->onPaint_before(this, Surface);

    draw(Surface);
    if (EPaint) EPaint->onPaint(this, Surface);

    draw_after(Surface);
    if (EPaint) EPaint->onPaint_after(this, Surface);


    #warning "Fix Gray Filter"
    //if (!enabled())
    //  Surface->filterGrayScale(rect<f32>::XYWH({}, {(f32)width(), (f32)height()}));

    remFlag(visDirtyDraw);
  }

  fun visual::doPaint_prepare(renderContext& Ctx) -> void
  {
    if (hasFlagR(visDirtyResize))
      doResize();

    if (hasFlagR(visDirtyReset))
      doReset(Ctx);

    if (hasFlagR(visDirtyRebound))
      m_endPoit = m_poit +m_size;
  }

  fun visual::doResize() -> void
  {
    if (EPaint) EPaint->onResize(this);
  }

  fun visual::doClick() -> void
  {
    if (EAction) EAction->onClick(this);
  }

  fun visual::doClickEx(poit<f32> Pos) -> void
  {
    if (EAction) EAction->onClickEx(this, Pos);
  }

  fun visual::doDblClick() -> void
  {
    if (EAction) EAction->onDblClick(this);
  }

  fun visual::doMouseDown(poit<f32> Pos, shiftStateSet Button, shiftStateSet State) -> void
  {
    if (EMouse) EMouse->onMouseDown(this, Pos, Button, State);
  }

  fun visual::doMouseUp(poit<f32> Pos, shiftStateSet Button, shiftStateSet State) -> void
  {
    if (EMouse) EMouse->onMouseUp(this, Pos, Button, State);


    // Click
    if (State != 0 || Button != shiftStates::ssLeft)
      return;

    if (
      Pos.X < 0      | Pos.Y < 0 ||
      Pos.X > size().W | Pos.Y > size().H
    )
      return;


    if (GetTickCount() -m_lastClick <= 300 && EAction && EAction->onDblClick) [[unlikely]]
    {
      doDblClick();

      m_lastClick = 0;
    }
    else [[likely]]
    {
      doClickEx(Pos);
      doClick();

      m_lastClick = GetTickCount();
    }

  }

  fun visual::doMouseMove(poit<f32> Pos, shiftStateSet State) -> void { if (EMouse) EMouse->onMouseMove(this, Pos, State); }
  fun visual::doMouseScrollVert(poit<f32> Pos, f32 Delta, shiftStateSet State) -> void { if (EMouse) EMouse->onMouseScrollVert(this, Pos, Delta, State); }
  fun visual::doMouseScrollHorz(poit<f32> Pos, f32 Delta, shiftStateSet State) -> void { if (EMouse) EMouse->onMouseScrollHorz(this, Pos, Delta, State); }

  fun visual::doTouchScrollVert(poit<f32> Pos, f32 Delta, shiftStateSet State)          -> void { if (ETouch) ETouch->onTouchScrollVert(this, Pos, Delta, State); }
  fun visual::doTouchScrollHorz(poit<f32> Pos, f32 Delta, shiftStateSet State)          -> void { if (ETouch) ETouch->onTouchScrollHorz(this, Pos, Delta, State); }
  fun visual::doTouchScrollVertFinish(poit<f32> Pos, f32 AvgSpeed, shiftStateSet State) -> void { if (ETouch) ETouch->onTouchScrollVertFinish(this, Pos, AvgSpeed, State); }
  fun visual::doTouchScrollHorzFinish(poit<f32> Pos, f32 AvgSpeed, shiftStateSet State) -> void { if (ETouch) ETouch->onTouchScrollHorzFinish(this, Pos, AvgSpeed, State); }
  fun visual::doTouchPinch(poit<f32> Delta, f32 Scale, shiftStateSet State)             -> void { if (ETouch) ETouch->onTouchPinch(this, Delta, Scale, State); }
  fun visual::doTouchRotate(poit<f32> Delta, f32 Rotate, shiftStateSet State)           -> void { if (ETouch) ETouch->onTouchRotate(this, Delta, Rotate, State); }

  fun visual::doKeyDown(u64 Key, u32 KeyCode, shiftStateSet State) -> void { if (EKey) EKey->onKeyDown(this, Key, KeyCode, State); }
  fun visual::doKeyUp  (u64 Key, u32 KeyCode, shiftStateSet State) -> void { if (EKey) EKey->onKeyUp(this, Key, KeyCode, State); }

  fun visual::doStateChanged(stateFlagSet State) -> void { onStateChanged(this, State); }
  
}
