from __future__ import print_function

import re
import sys

from jinja2 import Environment


def delimiter(component_statement):
    if component_statement[-1] != '}':
        return ';'
    else:
        return ''

def component_name(component_statement):
    m = re.match(r'\w+\s+(\w+)', component_statement)
    if not m:
        print("Invalid component statement: '%s'" % component_statement)
        exit(1)
    return m.group(1)

def to_snake_case(camel_case_text):
    s1 = re.sub('(.)([A-Z][a-z]+)', r'\1_\2', camel_case_text)
    return re.sub('([a-z0-9])([A-Z])', r'\1_\2', s1).lower()

tests = r'''
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn clear() {
        let mut ecm = ComponentManager::new();
        ecm.new_entity();
        ecm.new_entity();
        assert_eq!(ecm.iter().len(), 2);
        assert_eq!(ecm.has_entity(ID(0)), true);
        assert_eq!(ecm.has_entity(ID(1)), true);
        assert_eq!(ecm.has_entity(ID(2)), false);
        for id in ecm.iter() {
            assert!(ecm.has_entity(id));
        }
        ecm.remove_all_entities();
        assert_eq!(ecm.has_entity(ID(0)), false);
        assert_eq!(ecm.has_entity(ID(1)), false);
        assert_eq!(ecm.iter().len(), 0);
        ecm.new_entity();
        ecm.new_entity();
        ecm.new_entity();
        assert_eq!(ecm.iter().len(), 3);
        assert_eq!(ecm.has_entity(ID(0)), false);
        assert_eq!(ecm.has_entity(ID(1)), false);
        assert_eq!(ecm.has_entity(ID(2)), true);
        assert_eq!(ecm.has_entity(ID(3)), true);
        assert_eq!(ecm.has_entity(ID(4)), true);
        assert_eq!(ecm.has_entity(ID(5)), false);
        assert_eq!(ecm.has_entity(ID(6)), false);
        for id in ecm.iter() {
            assert!(ecm.has_entity(id));
        }
    }

    #[test]
    fn entity_id_on_add() {
        let mut ecm = ComponentManager::new();
        let e1_id = ecm.new_entity();
        let e2_id = ecm.new_entity();
        assert!(ecm.has_entity(e1_id));
        assert_eq!(e1_id, ID(0));
        assert!(ecm.has_entity(e2_id));
        assert_eq!(e2_id, ID(1));
    }

    #[test]
    fn remove_entity() {
        let mut ecm = ComponentManager::new();
        let e1_id = ecm.new_entity();
        let e2_id = ecm.new_entity();
        assert_eq!(ecm.has_entity(e1_id), true);
        assert_eq!(ecm.has_entity(e2_id), true);
        ecm.take_out(e1_id);
        assert_eq!(ecm.has_entity(e1_id), false);
        assert_eq!(ecm.has_entity(e2_id), true);
    }

    #[test]
    fn add_component() {
       let mut ecm = ComponentManager::new();
       let e = ecm.new_entity();
       assert_eq!(ecm.has_test_struct_component(e), false);
       assert_eq!(ecm.has_test_tuple_component(e), false);
       assert_eq!(ecm.has_test_enum_component(e), false);
       ecm.set_test_struct_component(e, TestStructComponent{x: 1, y: 2});
       assert_eq!(ecm.has_test_struct_component(e), true);
       assert_eq!(ecm.has_test_tuple_component(e), false);
       assert_eq!(ecm.has_test_enum_component(e), false);
       assert_eq!(ecm.get_test_struct_component(e), TestStructComponent{x: 1, y: 2});
    }
}
'''


if __name__ == '__main__':
    is_test = len(sys.argv) == 1
    if is_test:
        component_lines = [
            'struct TestStructComponent{x: int, y: int}',
            'struct TestTupleComponent(int, int)',
            'enum TestEnumComponent{A, B, C}',
        ]
    elif len(sys.argv) == 2:
        input_path = sys.argv[1]
        with open(input_path, 'r') as f:
            (component_section, template_section) = f.read().split('---\n')
    else:
        print("You must pass one or zero arguments")
        exit(1)


    definitions = [l.strip() + delimiter(l.strip())
                            for l in component_section.split('\n')
                            if l.strip()]
    components = [component_name(s) for s in definitions]

    environment = Environment()
    environment.filters['ident'] = to_snake_case
    template = environment.from_string(template_section)
    print(template.render(definitions=definitions, components=components))
