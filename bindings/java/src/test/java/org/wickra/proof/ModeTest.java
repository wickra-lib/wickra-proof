package org.wickra.proof;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import java.util.Map;
import java.util.TreeMap;
import java.util.stream.Stream;
import org.junit.jupiter.api.Test;

// Operating-mode equivalence: the proof does not depend on how its inputs
// arrive. A binding hands JSON text to the core. It can pass the committed
// golden bytes through untouched, or it can hand over what the host's JSON
// library re-emits -- keys in another order, whitespace, floats re-formatted.
// The core canonicalizes before hashing, so both operating modes must produce
// the same proof, byte for byte, and a proof re-emitted by the host must still
// verify. This is where a binding that mangles a float is caught.
//
// The binding carries no JSON library, so the test brings the smallest one
// that behaves like a host's: integers stay integral, floats go through
// Double.toString, objects come back with their keys in reverse order and the
// output is indented.
class ModeTest {
    private static Path findGolden() {
        Path dir = Path.of("").toAbsolutePath();
        for (int i = 0; i < 10 && dir != null; i++) {
            Path g = dir.resolve("golden");
            if (Files.isDirectory(g.resolve("specs"))) {
                return g;
            }
            dir = dir.getParent();
        }
        return null;
    }

    // -- a host-style JSON reader ------------------------------------------

    private static final class Reader {
        private final String text;
        private int at;

        Reader(String text) {
            this.text = text;
        }

        Object read() {
            Object value = value();
            skipWhitespace();
            if (at != text.length()) {
                throw new IllegalStateException("trailing text at " + at);
            }
            return value;
        }

        private void skipWhitespace() {
            while (at < text.length() && Character.isWhitespace(text.charAt(at))) {
                at++;
            }
        }

        private char expect(char c) {
            skipWhitespace();
            if (at >= text.length() || text.charAt(at) != c) {
                throw new IllegalStateException("expected '" + c + "' at " + at);
            }
            return text.charAt(at++);
        }

        private Object value() {
            skipWhitespace();
            char c = text.charAt(at);
            if (c == '{') {
                return object();
            }
            if (c == '[') {
                return array();
            }
            if (c == '"') {
                return string();
            }
            if (text.startsWith("true", at)) {
                at += 4;
                return Boolean.TRUE;
            }
            if (text.startsWith("false", at)) {
                at += 5;
                return Boolean.FALSE;
            }
            if (text.startsWith("null", at)) {
                at += 4;
                return null;
            }
            return number();
        }

        // Objects come back with their keys in reverse order: the opposite of
        // the canonical (sorted) form, and not the committed order either.
        private Map<String, Object> object() {
            Map<String, Object> out = new TreeMap<>(Collections.reverseOrder());
            expect('{');
            skipWhitespace();
            if (text.charAt(at) == '}') {
                at++;
                return out;
            }
            while (true) {
                skipWhitespace();
                String key = string();
                expect(':');
                out.put(key, value());
                skipWhitespace();
                if (text.charAt(at) == ',') {
                    at++;
                    continue;
                }
                expect('}');
                return out;
            }
        }

        private List<Object> array() {
            List<Object> out = new ArrayList<>();
            expect('[');
            skipWhitespace();
            if (text.charAt(at) == ']') {
                at++;
                return out;
            }
            while (true) {
                out.add(value());
                skipWhitespace();
                if (text.charAt(at) == ',') {
                    at++;
                    continue;
                }
                expect(']');
                return out;
            }
        }

        private String string() {
            expect('"');
            StringBuilder sb = new StringBuilder();
            while (true) {
                char c = text.charAt(at++);
                if (c == '"') {
                    return sb.toString();
                }
                if (c == '\\') {
                    char e = text.charAt(at++);
                    switch (e) {
                        case 'n' -> sb.append('\n');
                        case 't' -> sb.append('\t');
                        case 'r' -> sb.append('\r');
                        case 'b' -> sb.append('\b');
                        case 'f' -> sb.append('\f');
                        case 'u' -> {
                            sb.append((char) Integer.parseInt(text.substring(at, at + 4), 16));
                            at += 4;
                        }
                        default -> sb.append(e);
                    }
                } else {
                    sb.append(c);
                }
            }
        }

        // An integral literal stays a long; anything with a fraction or an
        // exponent becomes a double -- the split every host library makes.
        private Number number() {
            int start = at;
            while (at < text.length() && "+-0123456789.eE".indexOf(text.charAt(at)) >= 0) {
                at++;
            }
            String literal = text.substring(start, at);
            if (literal.indexOf('.') < 0 && literal.indexOf('e') < 0 && literal.indexOf('E') < 0) {
                return Long.parseLong(literal);
            }
            return Double.parseDouble(literal);
        }
    }

    // -- a host-style JSON writer ------------------------------------------

    private static void write(StringBuilder sb, Object value, int indent) {
        if (value == null) {
            sb.append("null");
        } else if (value instanceof String s) {
            sb.append('"');
            for (char c : s.toCharArray()) {
                switch (c) {
                    case '"' -> sb.append("\\\"");
                    case '\\' -> sb.append("\\\\");
                    case '\n' -> sb.append("\\n");
                    case '\r' -> sb.append("\\r");
                    case '\t' -> sb.append("\\t");
                    default -> {
                        if (c < 0x20) {
                            sb.append(String.format("\\u%04x", (int) c));
                        } else {
                            sb.append(c);
                        }
                    }
                }
            }
            sb.append('"');
        } else if (value instanceof Map<?, ?> map) {
            if (map.isEmpty()) {
                sb.append("{}");
                return;
            }
            sb.append("{\n");
            boolean first = true;
            for (Map.Entry<?, ?> e : map.entrySet()) {
                if (!first) {
                    sb.append(",\n");
                }
                first = false;
                sb.append("  ".repeat(indent + 1));
                write(sb, e.getKey(), indent + 1);
                sb.append(": ");
                write(sb, e.getValue(), indent + 1);
            }
            sb.append('\n').append("  ".repeat(indent)).append('}');
        } else if (value instanceof List<?> list) {
            if (list.isEmpty()) {
                sb.append("[]");
                return;
            }
            sb.append("[\n");
            boolean first = true;
            for (Object item : list) {
                if (!first) {
                    sb.append(",\n");
                }
                first = false;
                sb.append("  ".repeat(indent + 1));
                write(sb, item, indent + 1);
            }
            sb.append('\n').append("  ".repeat(indent)).append(']');
        } else {
            // Long, Double, Boolean: the host's own formatting (1000.0, 0.95, 1.0E-5).
            sb.append(value);
        }
    }

    private static String emit(Object value) {
        StringBuilder sb = new StringBuilder();
        write(sb, value, 0);
        return sb.toString();
    }

    private static Object decoded(String text) {
        return new Reader(text).read();
    }

    // The reader keeps `{"cmd": ..}` in reverse order too, so build the
    // envelope through it rather than by hand.
    private static Map<String, Object> envelope(String cmd) {
        Map<String, Object> out = new TreeMap<>(Collections.reverseOrder());
        out.put("cmd", cmd);
        return out;
    }

    @Test
    void committedAndHostSerialisedInputsProveAlike() throws IOException {
        Path golden = findGolden();
        assertNotNull(golden, "golden fixtures not found above the module");

        String dataText = Files.readString(golden.resolve("data.json")).strip();
        try (Stream<Path> stream = Files.list(golden.resolve("specs")); Prover prover = new Prover()) {
            List<Path> specs = stream.filter(p -> p.toString().endsWith(".json")).sorted().toList();
            assertFalse(specs.isEmpty(), "no golden specs; this would test nothing");
            for (Path specPath : specs) {
                String name = specPath.getFileName().toString();
                String specText = Files.readString(specPath).strip();
                String expected = Files.readString(golden.resolve("expected").resolve(name)).strip();

                // Mode 1: the committed bytes, spliced into the envelope verbatim.
                String committed = prover.command(
                        "{\"cmd\":\"prove\",\"spec\":" + specText + ",\"data\":" + dataText + "}");
                // Mode 2: what the host re-emits -- reversed key order, indented.
                Map<String, Object> hostedCmd = envelope("prove");
                hostedCmd.put("spec", decoded(specText));
                hostedCmd.put("data", decoded(dataText));
                String hosted = prover.command(emit(hostedCmd));
                assertEquals(expected, committed.strip(), "committed bytes: " + name);
                assertEquals(expected, hosted.strip(), "host-serialised bytes: " + name);

                // A proof the host re-emitted still verifies.
                Map<String, Object> verifyCmd = envelope("verify");
                verifyCmd.put("proof", decoded(expected));
                verifyCmd.put("spec", decoded(specText));
                verifyCmd.put("data", decoded(dataText));
                assertEquals("{\"ok\":true,\"valid\":true}", prover.command(emit(verifyCmd)).strip(), name);
            }
        }
    }
}
