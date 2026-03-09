#!/usr/bin/python3

import re
import sys

def split_qcl_params(param_str):
	"""
	Template (<>) ve parantez (()) içindeki virgülleri yoksayarak
	parametreleri güvenli bir şekilde böler.
	"""
	params = []
	current = ""
	angle_depth = 0
	paren_depth = 0

	for char in param_str:
		if char == '<':
			angle_depth += 1
		elif char == '>':
			angle_depth -= 1
		elif char == '(':
			paren_depth += 1
		elif char == ')':
			paren_depth -= 1
		elif char == ',' and angle_depth == 0 and paren_depth == 0:
			params.append(current.strip())
			current = ""
			continue
		
		current += char

	if current.strip():
		params.append(current.strip())
		
	return params

def parse_qcl_headers(input_file, output_file, inc_file):
	try:
		with open(input_file, 'r', encoding='utf-8') as f:
			lines = f.readlines()
	except FileNotFoundError:
		print(f"Hata: {input_file} bulunamadı.")
		return

	components = {}
	in_qcl_object = False
	current_object = ""
	brace_level = 0
	in_event_block = False

	obj_pattern  = re.compile(r'(?:struct\s+)?qcl_object\s+(\w+)(?:\s*:\s*(?:public\s+|protected\s+|private\s+)?([a-zA-Z0-9_:]+))?')
	prop_pattern = re.compile(r'qcl_prop\s*\((.*?)\)')
	func_pattern = re.compile(r'qcl_func\s+qev<(.*?)>\s+(\w+)\s*;')
	event_start_pattern = re.compile(r'struct\s+(event_\w+)\s*:\s*event_')
	event_end_pattern = re.compile(r'\}\s*\*(\w+)')

	namespace_pattern = re.compile(r'namespace\s+([a-zA-Z0-9_:]+)')
	global_brace_level = 0
	active_namespaces = []

	for line_num, line in enumerate(lines, 1):
		clean_line = re.sub(r'//.*', '', line)

		# Namespace takibi
		if "namespace" in clean_line and "using" not in clean_line:
			ns_match = namespace_pattern.search(clean_line)
			if ns_match:
				active_namespaces.append((ns_match.group(1), global_brace_level))

		global_brace_level += clean_line.count('{')
		global_brace_level -= clean_line.count('}')

		while active_namespaces and global_brace_level < active_namespaces[-1][1]:
			active_namespaces.pop()

		if not in_qcl_object:
			obj_match = obj_pattern.search(line)
			if obj_match:
				in_qcl_object = True
				base_name = obj_match.group(1)
				parent = obj_match.group(2) if obj_match.lastindex >= 2 else None
				current_object = "::".join([ns[0] for ns in active_namespaces] + [base_name])
				components[current_object] = {"props": [], "funcs": [], "parent": parent}
				brace_level = clean_line.count('{') - clean_line.count('}')
			continue

		brace_level += line.count('{')
		brace_level -= line.count('}')
		if brace_level <= 0 and '}' in line:
			in_qcl_object = False
			continue

		if event_start_pattern.search(line):
			in_event_block = True
			continue

		ee_match = event_end_pattern.search(line)
		if ee_match:
			ptr_name = ee_match.group(1)
			for p in components[current_object]['props']:
				if p.get('_temp'): p['EVENT_BLOCK'] = ptr_name; del p['_temp']
			for f in components[current_object]['funcs']:
				if f.get('_temp'): f['EVENT_BLOCK'] = ptr_name; del f['_temp']
			in_event_block = False
			continue

		prop_match = prop_pattern.search(line)
		if prop_match:
			# Virgülle ayırma işlemini yeni fonksiyona devrettik
			params = split_qcl_params(prop_match.group(1))
			
			p_dict = {"NAME": "", "TYPE": "", "READ": "", "WRITE": "", "PROP": "", "NEEDS_FUNC": False}
			for param in params:
				if param.startswith('PROP FUNC '):
					p_dict['PROP'] = param.replace('PROP FUNC ', '').strip()
					p_dict['NEEDS_FUNC'] = True
				elif ' ' in param:
					k, v = param.split(' ', 1)
					p_dict[k.strip()] = v.strip()
			if in_event_block: p_dict['_temp'] = True
			components[current_object]['props'].append(p_dict)

		func_match = func_pattern.search(line)
		if func_match:
			f_dict = {"NAME": func_match.group(2).strip()}
			if in_event_block: f_dict['_temp'] = True
			components[current_object]['funcs'].append(f_dict)

	with open(output_file, 'w', encoding='utf-8') as out:
		out.write("// Auto generated file, QCL Meta Compiler\n")
		out.write("#include <string_view>\n#include <any>\n#include <expected>\n")
		out.write("#include \"qcl/Types.hh\"\n#include \"DS.hh\"\n")
		out.write(f"#include \"{inc_file}\"\n\n")

		out.write("""\n\n
namespace {
  using hash_t = uint64_t;

  constexpr hash_t hash_fnv1a(std::string_view str) {
    hash_t hash = 0xcbf29ce484222325ULL;
    for (char c: str)
      hash = (hash ^ static_cast<hash_t>(c)) * 0x100000001b3ULL;

    return hash;
  }

  constexpr hash_t operator ""_h(const char* str, size_t len) {
    return hash_fnv1a(std::string_view(str, len));
  }
}
\n\n""")

		for comp, data in components.items():
			parent = data['parent']
			# getProp
			out.write(f"auto {comp}::getProp(std::string_view Name) -> std::expected<std::any, bool> {{\n")
			if parent: out.write(f"  if (auto Res = {parent}::getProp(Name); Res) return std::unexpected(Res.error());\n")
			out.write("  switch (hash_fnv1a(Name)) {\n")
			for p in data['props']:
				if p['READ']:
					ptr = p.get('EVENT_BLOCK')
					call = f"{ptr}->{p['READ']}()" if ptr else f"{p['READ']}()"
					init = f"event_enabled<true>({ptr}); " if ptr else ""
					out.write(f"    case \"{p['NAME']}\"_h: {{ {init}return {call}; }}\n")
			out.write("    default: return std::unexpected(true);\n  }\n}\n\n")

			# setProp
			out.write(f"auto {comp}::setProp(std::string_view Name, std::any Prop) -> std::expected<void, bool> {{\n")
			if parent: out.write(f"  if (auto Res = {parent}::setProp(Name, Prop); Res) return Res;\n")
			out.write("  switch (hash_fnv1a(Name)) {\n")
			for p in data['props']:
				if p['WRITE']:
					ptr = p.get('EVENT_BLOCK')
					call = f"{ptr}->{p['WRITE']}" if ptr else p['WRITE']
					init = f"event_enabled<true>({ptr}); " if ptr else ""
					out.write(f"    case \"{p['NAME']}\"_h: {{ {init}{call}(std::any_cast<{p['TYPE']}>(Prop)); return {{}}; }}\n")
			out.write("    default: return std::unexpected(true);\n  }\n}\n\n")

			# loadProp
			out.write(f"auto {comp}::loadProp(std::string_view Name, const qcl::ds::value& Prop, std::function<std::expected<qcl::qev_seed,bool>(std::string_view)> FuncMap) -> std::expected<bool, std::string> {{\n")
			if parent: out.write(f"  if (auto Res = {parent}::loadProp(Name, Prop, FuncMap); !Res || *Res) return Res;\n")
			out.write("  switch (hash_fnv1a(Name)) {\n")
			for p in data['props']:
				if p['PROP']:
					ptr = p.get('EVENT_BLOCK')
					init = f"event_enabled<true>({ptr}); " if ptr else ""
					args = "Prop, FuncMap" if p['NEEDS_FUNC'] else "Prop"
					out.write(f"    case \"{p['NAME']}\"_h: {{ {init}return {p['PROP']}({args}); }}\n")
			for f in data['funcs']:
				ptr = f.get('EVENT_BLOCK')
				init = f"event_enabled<true>({ptr}); " if ptr else ""
				target = f"{ptr}->{f['NAME']}" if ptr else f['NAME']
				out.write(f"    case \"{f['NAME']}\"_h: {{ if (!Prop.isString()) return std::unexpected(\"string expected\"); auto Seed = FuncMap(Prop.w_string()); if (!Seed) return std::unexpected(\"func not found\"); {init}Seed->ToLoad({target}); return true; }}\n")
			out.write("    default: return false;\n  }\n}\n")

if __name__ == "__main__":
	parse_qcl_headers(sys.argv[1], sys.argv[2], sys.argv[3])