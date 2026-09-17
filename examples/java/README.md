# Wickra Proof examples — Java

Runnable Java examples for the [Wickra Proof Java binding](../../bindings/java). The binding reaches the C ABI
through the Foreign Function & Memory API (JDK 22+), so build the library once
and point the JVM at it with `-Dnative.lib.dir`:

```bash
cargo build -p wickra-proof-c --release
```

## Run

As the CI examples job runs it, from the repository root:

```bash
mvn -f bindings/java/pom.xml -q package -DskipTests
javac -cp bindings/java/target/classes examples/java/Prove.java -d examples/java/out
java --enable-native-access=ALL-UNNAMED  -Dnative.lib.dir="$PWD/target/release"  -cp "bindings/java/target/classes:examples/java/out" Prove
```

## The examples

| Example | What it does |
|---------|--------------|
| `Prove.java` | A runnable Java example: prove a (spec, data) pair through the binding, print the report hash, then verify the proof and assert it holds. |
