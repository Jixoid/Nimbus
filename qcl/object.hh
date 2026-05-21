/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


#pragma once

#include <any>
#include <cassert>
#include <concepts>
#include <expected>
#include <functional>
#include <string_view>
#include <vector>
#include <sys/time.h>
#include "qcl/ds/ds.hh"
#include "qcl/types.hh"

using namespace std;

#define qcl_object
#define qcl_prop(...)
#define qcl_func



namespace qcl
{
  struct visual;    // Forward
  struct widget;    // Forward
  struct view;      // Forward


  /**
   * @brief Base class for all QCL objects.
   * 
   * Provides fundamental object properties such as parent-child hierarchy,
   * naming, styling, dynamic property management, and basic event handling.
   */
  struct qcl_object object
  {
    public:
      friend struct view;

    public:
      /**
       * @brief Default constructor.
       */
      inline constexpr object() {}

      /**
       * @brief Virtual destructor.
       */
      inline constexpr virtual ~object() {}
      

    private:
      view *m_parent{};

      string m_name{};
      
      vector<string> m_styles{};

      ds::value m_qdl{ds::value::makeUnDef()};
      
      u0 m_tag{};

    public:
      /**
       * @brief Gets the parent view of the object.
       * @return Pointer to the parent view.
       */
      inline fun parent() const noexcept -> view* {return m_parent;}

      /**
       * @brief Sets the parent view of the object.
       * @param Parent Pointer to the new parent view.
       */
      fun setParent(view* Parent) -> void;

      /**
       * @brief Gets the object's name.
       * @return The object's name.
       */
      inline fun name() noexcept -> string {return m_name;}

      /**
       * @brief Sets the object's name.
       * @param Val The new name for the object.
       */
      inline fun setName(string Val) noexcept -> void {m_name = Val;}
      qcl_prop(TYPE std::string, NAME name, READ name, WRITE setName);
      
      /**
       * @brief Gets the object's tag.
       * @return The current tag value.
       */
      inline fun tag() const noexcept -> u0 {return m_tag;}

      /**
       * @brief Sets the object's tag.
       * @param Val The new tag value.
       */
      inline fun setTag(u0 Val) noexcept -> void {m_tag = Val;}

      /**
       * @brief Sets the object's tag from a dynamic property value.
       * @param Prop The property containing the tag value.
       * @return An expected boolean indicating success, or an error message string on failure.
       */
      inline fun propTag(const ds::value& Prop) -> std::expected<bool, std::string> {
        if (!Prop.isInt()) return std::unexpected("was expected int");

        setTag(Prop.w_int());
        return true;
      }
      qcl_prop(TYPE u0, NAME tag, READ tag, WRITE setTag, PROP propTag);

      /**
       * @brief Gets the list of styles applied to the object.
       * @return Reference to the vector of style names.
       */
      inline fun styles() -> vector<string>& {return m_styles;}

      /**
       * @brief Gets the QDL (QAOS Data Language) value associated with the object.
       * @return The QDL value.
       */
      inline fun qdl() -> ds::value {return m_qdl.nref();}

      /**
       * @brief Sets the QDL value for the object.
       * @param Val The new QDL value.
       */
      inline fun setQDL(ds::value& Val) noexcept -> void {m_qdl = Val.nref();}

    protected:
      /**
       * @brief Base structure for event handling within objects.
       */
      struct event_ {};

      /**
       * @brief Manages the lifecycle of an event instance based on the Active template parameter.
       * @tparam Active Boolean indicating whether the event should be active.
       * @tparam Type The specific event type, derived from event_.
       * @param Inst Reference to the event instance pointer.
       */
      template <bool Active, typename Type>
        requires std::derived_from<Type, event_>
      inline fun constexpr event_enabled(Type*& Inst) -> void {
        if constexpr (Active) {
          if (!Inst) Inst = new Type();
        }
        else {
          if (Inst) delete Inst; Inst = nil;
        }
      }

    public:
      /**
       * @brief Retrieves the root view in the object hierarchy.
       * @return Pointer to the root view.
       */
      fun getRoot() noexcept -> view*;

    public:
      /**
       * @brief Gets a property value dynamically by name.
       * @param Name The name of the property.
       * @return An expected any containing the property value, or false on failure.
       */
      virtual fun getProp(string_view Name) -> expected<any, bool>;

      /**
       * @brief Sets a property value dynamically by name.
       * @param Name The name of the property.
       * @param Prop The new value for the property.
       * @return An expected void on success, or false on failure.
       */
      virtual fun setProp(string_view Name, any Prop) -> expected<void, bool>;

      /**
       * @brief Loads a property dynamically from a ds::value.
       * @param Name The name of the property.
       * @param Prop The ds::value containing the property data.
       * @param EventResolver Function to resolve event seeds.
       * @return An expected boolean indicating success, or an error string on failure.
       */
      virtual fun loadProp(string_view Name, const ds::value& Prop, function<expected<qev_seed,bool>(string_view)> EventResolver) -> expected<bool, string>;
  };

}
