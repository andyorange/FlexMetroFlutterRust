// Rust Musical Timer Interface for Flutter
// This demonstrates the architecture where ALL beat logic is in Rust
// and Flutter only displays the results.

import 'rust/api/simple.dart' as rust_api;
import 'rust/api/flutter_interface.dart';

class RustMusicalTimer {
  /// Initialize the Rust timer system
  static String initFlutterTimer() {
    final result = rust_api.initFlutterTimer();
    print("🦀 $result");
    return result;
  }

  /// Start the timer with a specific bar configuration and tempo range
  static String startTimerWithConfig(List<List<int>> bars, double startTempo, double endTempo) {
    // Convert List<List<int>> to List<(int, int)>
    final barConfigs = bars.map((bar) => (bar[0], bar[1])).toList();
    
    final result = rust_api.startTimerWithConfig(
      barConfigs: barConfigs,
      startTempo: startTempo,
      endTempo: endTempo,
    );
    
    print("🦀 $result");
    return result;
  }

  /// Stop the timer
  static String stopTimer() {
    final result = rust_api.stopTimer();
    print("🦀 $result");
    return result;
  }

  /// Get the latest beat events from Rust
  static List<FlutterBeatEvent> getBeatEvents() {
    return rust_api.getBeatEvents();
  }
}
