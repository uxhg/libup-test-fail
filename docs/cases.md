## Prerequisite
+ You need JDK and Maven to reproduce.
+ Cases are tested with Java 8 if not specified.
+ You can reproduce by upgrading specified library by editing `pom.xml`.

## Case: rtree/com.esotericsoftware:kryo
+ client: rtree
   + sha: 7c6f20019d46ec21ea5df97597f1dea592a115e4
   + url: https://github.com/davidmoten/rtree
   + tag: 0.8.6

+ library: com.esotericsoftware:kryo
	+ old version: 3.0.3
	+ new version: 5.0.0-RC4

+ Failing test:
```
Tests in error:
  testKryo(com.github.davidmoten.rtree.KryoSerializationTest): Class is not registered: com.github.davidmoten.rtree.KryoSerializationTest$Boo(..)
```

+ Reproduce: `mvn test -Dtest=com.github.davidmoten.rtree.KryoSerializationTest` or just `mvn test`
  to run all the tests.

+ Root cause: see https://github.com/EsotericSoftware/kryo/issues/398#issuecomment-371153541

+ Fix 1: Add `setRegistrationRequired(false)` before the call to `writeOutput()`.

```diff
--- a/src/test/java/com/github/davidmoten/rtree/KryoSerializationTest.java
+++ b/src/test/java/com/github/davidmoten/rtree/KryoSerializationTest.java
@@ -40,2 +40,3 @@ public class KryoSerializationTest {
         Boo b = new Boo("hello");
+        kryo.setRegistrationRequired(false);
         kryo.writeObject(output, b);
```

+ Another fix: register the class before the call to `writeOutput()`.

```diff
--- a/src/test/java/com/github/davidmoten/rtree/KryoSerializationTest.java
+++ b/src/test/java/com/github/davidmoten/rtree/KryoSerializationTest.java
@@ -40,2 +40,3 @@ public class KryoSerializationTest {
         Boo b = new Boo("hello");
+        kryo.register(Boo.class);
         kryo.writeObject(output, b);
```

## Case: reflectasm/org.ow2.asm:asm
+ client: reflectasm
  - at tag: reflectasm-1.11.8
  - sha1: 7cab65bb46ccd17b07ad05dd6b83d92a695477fc
  - url: https://github.com/EsotericSoftware/reflectasm

+ lib: org.ow2.asm:asm,
  - old version: 5.1,
  - new version: 7.2,

+ type: Error Only

+ Failing tests:
Multiple tests in `com.esotericsoftware.reflectasm.MethodAccessTest`.
```
[ERROR] Tests run: 3, Failures: 0, Errors: 2, Skipped: 0, Time elapsed: 0.032 s <<< FAILURE! - in com.esotericsoftware.reflectasm.MethodAccessTest
[ERROR] com.esotericsoftware.reflectasm.MethodAccessTest.testInvokeInterface  Time elapsed: 0.016 s  <<< ERROR!
java.lang.IllegalArgumentException: Class versions V1_5 or less must use F_NEW frames.
        at com.esotericsoftware.reflectasm.MethodAccessTest.testInvokeInterface(MethodAccessTest.java:93)

[ERROR] com.esotericsoftware.reflectasm.MethodAccessTest.testInvoke  Time elapsed: 0 s  <<< ERROR!
java.lang.IllegalArgumentException: Class versions V1_5 or less must use F_NEW frames.
        at com.esotericsoftware.reflectasm.MethodAccessTest.testInvoke(MethodAccessTest.java:25)
```

+ Root cause: new version throws an exception if API `visitFrame` is used incorrectly for old class versions.
  - check https://gitlab.ow2.org/asm/asm/-/issues/317872 and
    https://gitlab.ow2.org/asm/asm/-/merge_requests/263/

+ Fix: Use `F_NEW` option. Note that this fix edits the source code instead of test code.

```diff
diff --git a/src/com/esotericsoftware/reflectasm/MethodAccess.java b/src/com/esotericsoftware/reflectasm/MethodAccess.java
index bd6456f..629dc8c 100644
--- a/src/com/esotericsoftware/reflectasm/MethodAccess.java
+++ b/src/com/esotericsoftware/reflectasm/MethodAccess.java
@@ -154,5 +154,5 @@ public abstract class MethodAccess {
                            if (i == 0)
-                               mv.visitFrame(Opcodes.F_APPEND, 1, new Object[] {classNameInternal}, 0, null);
+                               mv.visitFrame(Opcodes.F_NEW, 1, new Object[] {classNameInternal}, 0, null);^M
                            else
-                               mv.visitFrame(Opcodes.F_SAME, 0, null, 0, null);
+                               mv.visitFrame(Opcodes.F_NEW, 0, null, 0, null);^M
                            mv.visitVarInsn(ALOAD, 4);
@@ -257,3 +257,3 @@ public abstract class MethodAccess {
                        mv.visitLabel(defaultLabel);
-                       mv.visitFrame(Opcodes.F_SAME, 0, null, 0, null);
+                       mv.visitFrame(Opcodes.F_NEW, 0, null, 0, null);^M
                    }
```


## flowable-engine/com.h2database:h2
+ client: flowable-engine
 - 'sha': '67466b4d620d6d4d29b6dee8ab52fb1cf367415a'
 - 'tag': 'flowable-6.4.1'
 - 'url': 'https://github.com/flowable/flowable-engine'

+ library: com.h2database:h2
  - old version: 1.4.197 
  - new version: 1.4.200

Upgrade by modifying `pom.xml`.
```diff
--- a/pom.xml
+++ b/pom.xml
@@ -84,3 +84,3 @@
                <artifactId>h2</artifactId>
-               <version>1.4.197</version>
+               <version>1.4.200</version>
            </dependency>
```

+ There are failures before upgrading, but we can just focus that specific test which does not fail before upgrading.

+ Failed test: `org.flowable.cmmn.test.jupiter.FlowableCmmnJupiterCustomResourceTest`, this test passed before upgrading.
```
[ERROR] Tests run: 1, Failures: 0, Errors: 1, Skipped: 0, Time elapsed: 1.278 s <<< FAILURE! - in org.flowable.cmmn.test.jupiter.FlowableCmmnJupiterCustomResourceTest
[ERROR] customResourceUsages{CmmnEngine}  Time elapsed: 1.276 s  <<< ERROR!
org.apache.ibatis.exceptions.PersistenceException:

### Error getting a new connection.  Cause: org.h2.jdbc.JdbcSQLNonTransientConnectionException: Unsupported connection setting "MVCC" [90113-200]
### Cause: org.h2.jdbc.JdbcSQLNonTransientConnectionException: Unsupported connection setting "MVCC" [90113-200]
```

Source code of the test:
```java
// src/test/java/org/flowable/cmmn/test/jupiter/FlowableCmmnJupiterCustomResourceTest.java
import org.flowable.cmmn.engine.test.CmmnConfigurationResource;
@FlowableCmmnTest
@CmmnConfigurationResource("flowable.custom.cmmn.cfg.xml")
class FlowableCmmnJupiterCustomResourceTest {
    @Test
    void customResourceUsages(CmmnEngine cmmnEngine) {
        assertThat(cmmnEngine.getName()).as("cmmn engine name").isEqualTo("customName");
    }
}
```

And in the specified configuration file, `MVCC` option is set to `TRUE`.
```xml
<!-- src/test/resources/flowable.custom.cmmn.cfg.xml -->
<property name="jdbcUrl" value="jdbc:h2:mem:flowable;DB_CLOSE_DELAY=1000;MVCC=TRUE"/>
```

+ Fix: remove the `MVCC=TRUE` configuration, see the patch below.


```diff
--- a/modules/flowable-cmmn-engine/src/test/resources/flowable.custom.cmmn.cfg.xml
+++ b/modules/flowable-cmmn-engine/src/test/resources/flowable.custom.cmmn.cfg.xml
@@ -7,3 +7,3 @@
     <bean id="cmmnEngineConfiguration" class="org.flowable.cmmn.engine.impl.cfg.StandaloneInMemCmmnEngineConfiguration">
-        <property name="jdbcUrl" value="jdbc:h2:mem:flowable;DB_CLOSE_DELAY=1000;MVCC=TRUE"/>
+        <property name="jdbcUrl" value="jdbc:h2:mem:flowable;DB_CLOSE_DELAY=1000"/>
         <property name="jdbcDriver" value="org.h2.Driver"/>
```

