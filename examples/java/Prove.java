// A runnable Java example: prove a (spec, data) pair through the binding, print
// the report hash, then verify the proof and assert it holds.
//
//   cargo build -p wickra-proof-c
//   mvn -f bindings/java/pom.xml -q package -DskipTests
//   javac -cp bindings/java/target/classes examples/java/Prove.java -d examples/java/out
//   java --enable-native-access=ALL-UNNAMED \
//        -Dnative.lib.dir=target/debug \
//        -cp "bindings/java/target/classes;examples/java/out" Prove
import org.wickra.proof.Prover;

public final class Prove {
    private static final String SPEC =
            "{\"strategy\":{\"symbol\":\"AAA\",\"timeframe\":\"1h\","
                    + "\"indicators\":{\"ema_fast\":{\"type\":\"Ema\",\"params\":[3]},"
                    + "\"ema_slow\":{\"type\":\"Ema\",\"params\":[8]}},"
                    + "\"entry\":{\"cross_above\":[\"ema_fast\",\"ema_slow\"]},"
                    + "\"exit\":{\"cross_below\":[\"ema_fast\",\"ema_slow\"]},"
                    + "\"sizing\":{\"type\":\"fixed_fraction\",\"fraction\":0.95},"
                    + "\"costs\":{\"taker_bps\":5,\"slippage\":{\"type\":\"fixed_bps\",\"bps\":2}},"
                    + "\"risk\":{}},\"dataset_ref\":\"example/AAA/1h\"}";

    // A short V-shaped price path so the fast/slow EMA cross fires at least once.
    private static final int[] CLOSES = {120, 118, 116, 114, 112, 110, 108, 112, 116, 120, 124, 128};

    private static String data() {
        StringBuilder sb = new StringBuilder("{\"AAA\":[");
        for (int i = 0; i < CLOSES.length; i++) {
            int close = CLOSES[i];
            int open = i == 0 ? close : CLOSES[i - 1];
            if (i > 0) {
                sb.append(',');
            }
            sb.append("{\"time\":").append(1_700_000_000L + i * 3600L)
                    .append(",\"open\":").append(open)
                    .append(",\"high\":").append(Math.max(open, close) + 1)
                    .append(",\"low\":").append(Math.min(open, close) - 1)
                    .append(",\"close\":").append(close)
                    .append(",\"volume\":1000}");
        }
        return sb.append("]}").toString();
    }

    public static void main(String[] args) {
        String data = data();
        try (Prover prover = new Prover()) {
            String proof = prover.command(
                    "{\"cmd\":\"prove\",\"spec\":" + SPEC + ",\"data\":" + data + "}");

            System.out.println("wickra-proof " + Prover.version());
            int hashAt = proof.indexOf("\"report_hash\":");
            if (hashAt >= 0) {
                System.out.println("proof: " + proof.substring(hashAt, hashAt + 30) + "...");
            }

            // The prove response is valid JSON, so it drops straight in as "proof".
            String verdict = prover.command(
                    "{\"cmd\":\"verify\",\"proof\":" + proof + ",\"spec\":" + SPEC
                            + ",\"data\":" + data + "}");
            if (!verdict.contains("\"valid\":true")) {
                throw new IllegalStateException("proof must verify, got: " + verdict);
            }
            System.out.println("verify: valid");
        }
    }
}
