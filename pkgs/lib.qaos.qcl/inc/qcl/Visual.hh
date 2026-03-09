/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/



#pragma once

#include <expected>
#include <string_view>
#include <vector>

#include "DS.hh"

#include "qcl/Object.hh"
#include "qcl/Effect.hh"
#include "qcl/Types.hh"

using namespace std;

#define ef else if
#define el else


namespace qcl
{
  struct visual;

  class visualPaddings
  {
    friend class visual;

    private:
      visual *m_owner{};
      i32 m_left{}, m_righ{}, m_top{}, m_bot{};

    public:
      visualPaddings(visual* nOwner): m_owner(nOwner) {}

    public:
      inline fun top() const noexcept -> const i32 {return m_top;}
      inline fun left() const noexcept -> const i32 {return m_left;}
      inline fun right() const noexcept -> const i32 {return m_righ;}
      inline fun bottom() const noexcept -> const i32 {return m_bot;}
      
      
      inline fun setTop(i32 Val) noexcept -> void {m_top = Val; update();}
      inline fun setLeft(i32 Val) noexcept -> void {m_left = Val; update();}
      inline fun setRight(i32 Val) noexcept -> void {m_righ = Val; update();}
      inline fun setBottom(i32 Val) noexcept -> void {m_bot = Val; update();}
      inline fun set(rect<i32> Val) noexcept -> void {m_top = Val.Y1, m_left = Val.X1; m_bot = Val.Y2; m_righ = Val.X2; update();}
  
    private:
      fun update() -> void;
  };


  struct qcl_object visual: object
  {
    public:
      friend struct view;


    public:
      enum messages: u32
      {
        vmPaint  = 1,
        vmResize = 2,
        vmReset  = 3,

        vmMouseDown = 4,
        vmMouseUp   = 5,
        vmMouseMove = 6,
        vmMouseScrollVert = 7,
        vmMouseScrollHorz = 8,

        vmTouchScrollVert = 9,
        vmTouchScrollHorz = 10,
        vmTouchScrollVertFinish = 11,
        vmTouchScrollHorzFinish = 12,
        vmTouchPinch = 13,
        vmTouchRotate = 14,

        vmKeyDown = 15,
        vmKeyUp = 16,

        vmPlatformSpecific = u32(-1),
      };


    public:
      enum dirtyFlags: u16 {
        visDirtyAutoSize = (u16)1 << 0,
        visDirtyResize   = (u16)1 << 1,
        visDirtyRebound  = (u16)1 << 2,
        visDirtyDraw     = (u16)1 << 3,

        visDirtyReset    = (u16)1 << 4,
        
        vieDirtyTiling   = (u16)1 << 5,
      };
      using dirtyFlagSet = u16;
      
      fun virtual addFlag(dirtyFlagSet Flag) noexcept -> void;
      fun virtual remFlag(dirtyFlagSet Flag) noexcept -> void;
      
      inline fun hasFlag (dirtyFlagSet Flag) const noexcept -> bool { return (m_dirtyFlags & Flag); }
      inline fun hasFlagR(dirtyFlagSet Flag) noexcept -> bool {
        const bool ret = (m_dirtyFlags & Flag);
        remFlag(Flag);
        return ret;
      }


    public:
      enum stateFlags: u16
      {
        sfNone   = 0,
        sfHover  = (u16)1 << 1,
        sfFocus  = (u16)1 << 2,
      };
      using stateFlagSet = u16;


    public:
      inline constexpr visual(): m_paddings(this) {}
      
      inline constexpr ~visual() override {
        for (auto &X: m_effects)
          delete X;
        m_effects.clear();

        if (EPaint)  delete EPaint;
        if (EAction) delete EAction;
        if (EMouse)  delete EMouse;
        if (ETouch)  delete ETouch;
        if (EKey)    delete EKey;
      }


    private:
      poit<i32> m_poit{0, 0};
      size<i32> m_size{20, 10};
      poit<i32> m_endPoit{m_poit+m_size};

      size<i32> m_preferedSize{};
      size<i32> m_minSize{};
      size<i32> m_maxSize{};

      vector<effect*> m_effects{};

      visualPaddings m_paddings;
      
      u64 m_lastClick{0};
      
      dirtyFlagSet m_dirtyFlags{visDirtyDraw | visDirtyAutoSize};
      stateFlagSet m_stateFlags{sfNone};
      
      bool m_autoSizeVert{false}, m_autoSizeHorz{false};

    protected:
      inline fun setPreferedSize(size<i32> Val) -> void {
        if (m_preferedSize == Val)
          return;
        
        m_preferedSize = Val;
        addFlag(visDirtyAutoSize);
      }

    public:
      inline fun point() const noexcept -> poit<i32> {return m_poit;}
      inline fun setPoint(poit<i32> Val) -> void { if (m_poit != Val) { m_poit = Val; update(visDirtyRebound); }}
      inline fun propPoint(const ds::value& Prop) -> std::expected<bool, std::string> {
        if (!Prop.isStruct()) return std::unexpected("was expected struct");
        
        auto Nat = point();
        
        for (int i{}; i < Prop.size(); i++) {
          const auto X = Prop.get_stc(i);

          if (X.first == "x") {
            if (!X.second.isInt()) return std::unexpected("was expected int");
            Nat.X = X.second.w_int();
          }
          ef (X.first == "y") {
            if (!X.second.isInt()) return std::unexpected("was expected int");
            Nat.Y = X.second.w_int();
          }
          else
            return std::unexpected("unexpected property: "+X.first);
        }

        setPoint(Nat);
        return true;
      }
      qcl_prop(TYPE poit<i32>, NAME point, READ point, WRITE setPoint, PROP propPoint);
      

      inline fun left() const noexcept -> i32 {return m_poit.X;}
      inline fun setLeft(i32 Val) -> void { if (m_poit.X != Val) { m_poit.X = Val; update(visDirtyRebound); }}
      inline fun propLeft(const ds::value& Prop) -> std::expected<bool, std::string> {
        if (!Prop.isInt()) return std::unexpected("was expected int");

        setLeft(Prop.w_int());
        return true;
      }
      qcl_prop(TYPE i32, NAME left, READ left, WRITE setLeft, PROP propLeft);
      

      inline fun top() const noexcept -> i32 {return m_poit.Y;}
      inline fun setTop(i32 Val) -> void { if (m_poit.Y != Val) { m_poit.Y = Val; update(visDirtyRebound); }}
      inline fun propTop(const ds::value& Prop) -> std::expected<bool, std::string> {
        if (!Prop.isInt()) return std::unexpected("was expected int");

        setTop(Prop.w_int());
        return true;
      }
      qcl_prop(TYPE i32, NAME top, READ top, WRITE setTop, PROP propTop);
      

      inline fun size() const noexcept -> size<i32> {return m_size;}
      inline virtual fun setSize(qcl::size<i32> Val) -> void { if (m_size != Val) { m_size = Val; update(visDirtyResize); } }
      inline fun propSize(const ds::value& Prop) -> std::expected<bool, std::string> {
        if (!Prop.isStruct()) return std::unexpected("was expected struct");
        
        auto Nat = size();
        
        for (int i{}; i < Prop.size(); i++) {
          const auto X = Prop.get_stc(i);

          if (X.first == "w") {
            if (!X.second.isInt()) return std::unexpected("was expected int");
            Nat.W = X.second.w_int();
          }
          ef (X.first == "h") {
            if (!X.second.isInt()) return std::unexpected("was expected int");
            Nat.H = X.second.w_int();
          }
          else
            return std::unexpected("unexpected property: "+X.first);
        }

        setSize(Nat);
        return true;
      }
      qcl_prop(TYPE qcl::size<i32>, NAME size, READ size, WRITE setSize, PROP propSize);


      inline fun width() const noexcept -> i32 {return m_size.W;}
      inline fun setWidth(i32 Val) -> void { if (m_size.W != Val) setSize({Val, m_size.H}); }
      inline fun propWidth(const ds::value& Prop) -> std::expected<bool, std::string> {
        if (!Prop.isInt()) return std::unexpected("was expected int");

        setWidth(Prop.w_int());
        return true;
      }
      qcl_prop(TYPE i32, NAME width, READ width, WRITE setWidth, PROP propWidth);


      inline fun height() const noexcept -> i32 {return m_size.H;}
      inline fun setHeight(i32 Val) -> void { if (m_size.H != Val) setSize({m_size.W, Val}); }
      inline fun propHeight(const ds::value& Prop) -> std::expected<bool, std::string> {
        if (!Prop.isInt()) return std::unexpected("was expected int");

        setHeight(Prop.w_int());
        return true;
      }
      qcl_prop(TYPE i32, NAME height, READ height, WRITE setHeight, PROP propHeight);


      inline fun endPoit() const noexcept -> const poit<i32> {return m_endPoit;}
      
      inline fun preferedSize() const noexcept -> const qcl::size<i32> {return m_preferedSize;}


      inline fun minSize() const noexcept -> qcl::size<i32> {return m_minSize;}
      inline fun setMinSize(qcl::size<i32> Val) -> void { if (m_minSize != Val) { m_minSize = Val; update(visDirtyRebound); }}
      inline fun propMinSize(const ds::value& Prop) -> std::expected<bool, std::string> {
        if (!Prop.isStruct()) return std::unexpected("was expected struct");
        
        auto Nat = size();
        
        for (int i{}; i < Prop.size(); i++) {
          const auto X = Prop.get_stc(i);

          if (X.first == "w") {
            if (!X.second.isInt()) return std::unexpected("was expected int");
            Nat.W = X.second.w_int();
          }
          ef (X.first == "h") {
            if (!X.second.isInt()) return std::unexpected("was expected int");
            Nat.H = X.second.w_int();
          }
          else
            return std::unexpected("unexpected property: "+X.first);
        }

        setMinSize(Nat);
        return true;
      }
      qcl_prop(TYPE qcl::size<i32>, NAME minSize, READ minSize, WRITE setMinSize, PROP propMinSize);


      inline fun maxSize() const noexcept -> qcl::size<i32> {return m_maxSize;}
      inline fun setMaxSize(qcl::size<i32> Val) -> void { if (m_maxSize != Val) { m_maxSize = Val; update(visDirtyRebound); }}
      inline fun propMaxSize(const ds::value& Prop) -> std::expected<bool, std::string> {
        if (!Prop.isStruct()) return std::unexpected("was expected struct");
        
        auto Nat = size();
        
        for (int i{}; i < Prop.size(); i++) {
          const auto X = Prop.get_stc(i);

          if (X.first == "w") {
            if (!X.second.isInt()) return std::unexpected("was expected int");
            Nat.W = X.second.w_int();
          }
          ef (X.first == "h") {
            if (!X.second.isInt()) return std::unexpected("was expected int");
            Nat.H = X.second.w_int();
          }
          else
            return std::unexpected("unexpected property: "+X.first);
        }

        setMaxSize(Nat);
        return true;
      }
      qcl_prop(TYPE qcl::size<i32>, NAME maxSize, READ maxSize, WRITE setMaxSize, PROP propMaxSize);


      inline fun autoSizeVert() const noexcept -> const bool {return m_autoSizeVert;}
      inline fun setAutoSizeVert(bool Val) -> void { if (m_autoSizeVert != Val) { m_autoSizeVert = Val; update(visDirtyRebound); }}
      inline fun propAutoSizeVert(const ds::value& Prop) -> std::expected<bool, std::string> {
        if (!Prop.isBool()) return std::unexpected("was expected bool");

        setAutoSizeVert(Prop.w_bool());
        return true;
      }
      qcl_prop(TYPE bool, NAME autoSizeVert, READ autoSizeVert, WRITE setAutoSizeVert, PROP propAutoSizeVert);


      inline fun autoSizeHorz() const noexcept -> const bool {return m_autoSizeHorz;}
      inline fun setAutoSizeHorz(bool Val) -> void { if (m_autoSizeHorz != Val) { m_autoSizeHorz = Val; update(visDirtyRebound); }}
      inline fun propAutoSizeHorz(const ds::value& Prop) -> std::expected<bool, std::string> {
        if (!Prop.isBool()) return std::unexpected("was expected bool");

        setAutoSizeHorz(Prop.w_bool());
        return true;
      }
      qcl_prop(TYPE bool, NAME autoSizeHorz, READ autoSizeHorz, WRITE setAutoSizeHorz, PROP propAutoSizeHorz);


      inline fun setAutoSize(bool Val) -> void { if (m_autoSizeVert != Val || m_autoSizeHorz != Val) { m_autoSizeVert = m_autoSizeHorz = Val; update(visDirtyRebound); }}
      inline fun propAutoSize(const ds::value& Prop) -> std::expected<bool, std::string> {
        if (!Prop.isBool()) return std::unexpected("was expected bool");

        setAutoSize(Prop.w_bool());
        return true;
      }
      qcl_prop(TYPE bool, NAME autoSize, WRITE setAutoSize, PROP propAutoSize);


      fun propPaddings(const ds::value& Prop) -> expected<bool, string>;
      inline fun paddings() noexcept -> visualPaddings& {return m_paddings;}
      qcl_prop(TYPE qcl::rect<i32>, NAME paddings, PROP propPaddings);

      
      inline fun effects() const noexcept -> vector<effect*> {return m_effects;}
      fun propEffects(const ds::value& Prop) -> expected<bool, string>;
      qcl_prop(TYPE vector<effect*>, NAME effects, READ effects, PROP propEffects);


      inline fun dirtyFlags() const noexcept -> dirtyFlagSet { return m_dirtyFlags; }
      inline fun stateFlags() const noexcept -> stateFlagSet { return m_stateFlags; }
      
    public:
      fun update(dirtyFlagSet Flag = visDirtyDraw) -> void;
      fun dyeToRoot(dirtyFlagSet Flag = visDirtyDraw) -> void;

      fun getLocalPos(view *Target) const noexcept -> poit<i32>;
    
      inline virtual fun draw_before(renderContext&) -> void {}
      inline virtual fun draw_after(renderContext&) -> void {}
      inline virtual fun draw(renderContext&) -> void {}

      fun addEffect(effect* Val) -> void { m_effects.push_back(Val); }
      fun remEffect(effect* Val) -> void { delete relEffect(Val); }
      [[nodiscard]] fun relEffect(effect*) -> effect*;

      virtual fun calcAutoSize(renderContext&) -> void;
      
    public:
      struct event_paint: event_ {
        qcl_func qev<void, renderContext& /* Canvas */> onPaint;
        qcl_func qev<void, renderContext& /* Canvas */> onPaint_before;
        qcl_func qev<void, renderContext& /* Canvas */> onPaint_after;
        qcl_func qev<void> onResize;
      } *EPaint{};

      struct event_action: event_ {
        qcl_func qev<void> onClick;
        qcl_func qev<void, poit<f32> /* Pos */> onClickEx;
        qcl_func qev<void> onDblClick;
      } *EAction{};

      struct event_mouse: event_ {
        qcl_func qev<void, poit<f32> /* Pos */, shiftStateSet /* Button */, shiftStateSet /* State */> onMouseDown;
        qcl_func qev<void, poit<f32> /* Pos */, shiftStateSet /* Button */, shiftStateSet /* State */> onMouseUp;
        qcl_func qev<void, poit<f32> /* Pos */, shiftStateSet /* State */> onMouseMove;
        qcl_func qev<void, poit<f32> /* Pos */, f32 /* Delta */, shiftStateSet /* State */ > onMouseScrollVert;
        qcl_func qev<void, poit<f32> /* Pos */, f32 /* Delta */, shiftStateSet /* State */ > onMouseScrollHorz;
      } *EMouse{};

      struct event_touch: event_ {
        qcl_func qev<void, poit<f32> /* Pos */, f32 /* Delta */, shiftStateSet /* State */ > onTouchScrollVert;
        qcl_func qev<void, poit<f32> /* Pos */, f32 /* Delta */, shiftStateSet /* State */ > onTouchScrollHorz;
        qcl_func qev<void, poit<f32> /* Pos */, f32 /* AvgSpeed */, shiftStateSet /* State */ > onTouchScrollVertFinish;
        qcl_func qev<void, poit<f32> /* Pos */, f32 /* AvgSpeed */, shiftStateSet /* State */ > onTouchScrollHorzFinish;
        qcl_func qev<void, poit<f32> /* Delta */, f32 /* Scale */, shiftStateSet /* State */> onTouchPinch;
        qcl_func qev<void, poit<f32> /* Delta */, f32 /* Rotate */, shiftStateSet /* State */> onTouchRotate;
      } *ETouch{};

      struct event_key: event_ {
        qcl_func qev<void, u64 /*Key*/, u32 /* KeyCode */, shiftStateSet /* State */> onKeyDown;
        qcl_func qev<void, u64 /*Key*/, u32 /* KeyCode */, shiftStateSet /* State */> onKeyUp;
      } *EKey{};

      qcl_func qev<void, stateFlagSet /* State */> onStateChanged;

    public:
      virtual fun handlerPaint (renderContext&) -> void; 
      virtual fun handlerResize() -> void;

      inline virtual fun handlerMouseDown       (poit<f32> Pos, shiftStateSet Button, shiftStateSet State) -> void { doMouseDown(Pos, Button, State); }
      inline virtual fun handlerMouseUp         (poit<f32> Pos, shiftStateSet Button, shiftStateSet State) -> void { doMouseUp(Pos, Button, State); }
      inline virtual fun handlerMouseMove       (poit<f32> Pos, shiftStateSet State)                       -> void { doMouseMove(Pos, State); }
      inline virtual fun handlerMouseScrollVert (poit<f32> Pos, f32 Delta, shiftStateSet State)            -> void { doMouseScrollVert(Pos, Delta, State); }
      inline virtual fun handlerMouseScrollHorz (poit<f32> Pos, f32 Delta, shiftStateSet State)            -> void { doMouseScrollHorz(Pos, Delta, State); }

      inline virtual fun handlerTouchScrollVert       (poit<f32> Pos, f32 Delta, shiftStateSet State)    -> void { doTouchScrollVert(Pos, Delta, State); }
      inline virtual fun handlerTouchScrollHorz       (poit<f32> Pos, f32 Delta, shiftStateSet State)    -> void { doTouchScrollHorz(Pos, Delta, State); }
      inline virtual fun handlerTouchScrollVertFinish (poit<f32> Pos, f32 AvgSpeed, shiftStateSet State) -> void { doTouchScrollVertFinish(Pos, AvgSpeed, State); }
      inline virtual fun handlerTouchScrollHorzFinish (poit<f32> Pos, f32 AvgSpeed, shiftStateSet State) -> void { doTouchScrollHorzFinish(Pos, AvgSpeed, State); }
      inline virtual fun handlerTouchPinch            (poit<f32> Delta, f32 Scale, shiftStateSet State)  -> void { doTouchPinch(Delta, Scale, State); }
      inline virtual fun handlerTouchRotate           (poit<f32> Delta, f32 Rotate, shiftStateSet State) -> void { doTouchRotate(Delta, Rotate, State); }

      inline virtual fun handlerKeyDown (u64 Key, u32 KeyCode, shiftStateSet State)                     -> void { doKeyDown(Key, KeyCode, State); };
      inline virtual fun handlerKeyUp   (u64 Key, u32 KeyCode, shiftStateSet State)                     -> void { doKeyUp(Key, KeyCode, State); };

      virtual fun handlerStateChanged (stateFlagSet State) -> void;

    public:
      virtual fun doReset         (renderContext&) -> void;
      virtual fun doPaint_prepare (renderContext&) -> void;
      virtual fun doPaint         (renderContext&) -> void;
      virtual fun doResize        () -> void;

      virtual fun doClick    () -> void;
      virtual fun doClickEx  (poit<f32> Pos) -> void;
      virtual fun doDblClick () -> void;

      virtual fun doMouseDown        (poit<f32> Pos, shiftStateSet Button, shiftStateSet State) -> void;
      virtual fun doMouseUp          (poit<f32> Pos, shiftStateSet Button, shiftStateSet State) -> void;
      virtual fun doMouseMove        (poit<f32> Pos, shiftStateSet State) -> void;
      virtual fun doMouseScrollVert  (poit<f32> Pos, f32 Delta, shiftStateSet State) -> void;
      virtual fun doMouseScrollHorz  (poit<f32> Pos, f32 Delta, shiftStateSet State) -> void;
      
      virtual fun doTouchScrollVert       (poit<f32> Pos, f32 Delta, shiftStateSet State) -> void;
      virtual fun doTouchScrollHorz       (poit<f32> Pos, f32 Delta, shiftStateSet State) -> void;
      virtual fun doTouchScrollVertFinish (poit<f32> Pos, f32 AvgSpeed, shiftStateSet State) -> void;
      virtual fun doTouchScrollHorzFinish (poit<f32> Pos, f32 AvgSpeed, shiftStateSet State) -> void;
      virtual fun doTouchPinch            (poit<f32> Delta, f32 Scale, shiftStateSet State) -> void;
      virtual fun doTouchRotate           (poit<f32> Delta, f32 Rotate, shiftStateSet State) -> void;

      virtual fun doKeyDown (u64 Key, u32 KeyCode, shiftStateSet State) -> void;
      virtual fun doKeyUp   (u64 Key, u32 KeyCode, shiftStateSet State) -> void;
      
      virtual fun doStateChanged (stateFlagSet State) -> void;

    public:
      fun getProp(string_view) -> expected<any, bool> override;
      fun setProp(string_view, any) -> expected<void, bool> override;
      fun loadProp(string_view, const ds::value&, function<expected<qev_seed,bool>(string_view)>) -> expected<bool, string> override;
  };

}
