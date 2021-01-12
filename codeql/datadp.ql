/**
 * @name Detecting dataflow involving multiple libraries
 * @description Dataflow involving multiple libraries can help us find potential conflicts between
        		dependencies
 * @id java/dataflow-dep
 * @kind path-problem
 */

import java
import DataFlow::PathGraph
import semmle.code.java.dataflow.DataFlow
import semmle.code.java.dataflow.TaintTracking

bindingset[j]
predicate notJavaLib(string j) { not j.regexpMatch("^java.*$") }

bindingset[jname1, jname2]
predicate notSameJar(string jname1, string jname2) {
  jname1.splitAt(".", 0) != jname2.splitAt(".", 0)
  or
  jname1.splitAt(".", 1) != jname2.splitAt(".", 1)
  or
  jname1.splitAt(".", 2) != jname2.splitAt(".", 2)
}

class DataDepLibCalls extends TaintTracking::Configuration {
  DataDepLibCalls() {
    // unique identifier for this configuration
    this = "DataDepLibCalls"
  }

  override predicate isSource(DataFlow::Node nd) {
    exists(Call libCall |
      notJavaLib(libCall.getCallee().getDeclaringType().getQualifiedName()) and
      nd.asExpr() = libCall
    )
  }

  override predicate isSink(DataFlow::Node nd) { exists(Expr e | nd.asExpr() = e) }
  // override predicate isSink(DataFlow::Node nd) {
  //     exists(Call libCall |
  //        notJavaLib(libCall.getCallee().getDeclaringType().getQualifiedName()) and
  //        nd.asExpr() = libCall.getAnArgument()
  //      )
  //    }
}

from DataDepLibCalls pt, Call source, Call sink, string lib1, string lib2, Expr e
where
  lib1 = source.getCallee().getDeclaringType().getQualifiedName() and
  lib2 = sink.getCallee().getDeclaringType().getQualifiedName() and
  notSameJar(lib1, lib2) and
  e = sink.getAnArgument() and
  pt.hasFlow(DataFlow::exprNode(source), DataFlow::exprNode(e))
select source, lib1, sink, lib2
