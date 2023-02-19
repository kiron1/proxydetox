import Foundation

public class ProxydetoxControl {
  static let autostartKey = "Autostart"
  static let portKey = "Port"
  static let pacFileKey = "PacFile"
  static let negotiateKey = "Negotiate"
  static let alwaysUseConnectKey = "alwaysUseConnect"
  static let directFallbackKey = "DirectFallback"

  private var internalNetworkAvailable: Bool = true

  // private var pd: OpaquePointer? = nil;
  // private var worker: Thread? = nil
  private var proxydetoxUrl: URL
  private var proxydetoxProcess: Process?

  init() {
    let path = Bundle.main.bundlePath as NSString
    var components = path.pathComponents
    components.append("Contents")
    components.append("MacOS")
    components.append("proxydetoxcli")
    self.proxydetoxUrl = URL(fileURLWithPath: NSString.path(withComponents: components))
    print("proxydetoxcli \(self.proxydetoxUrl)")

    let applicationSupportURL = FileManager.default.urls(
      for: .applicationSupportDirectory, in: .userDomainMask
    ).first
    print("applicationSupportURL \(applicationSupportURL)")

    var proxyPacPath = FileManager.default.urls(
      for: .applicationSupportDirectory, in: .userDomainMask
    ).first!
    proxyPacPath.appendPathComponent("Proxydetox")

    if !FileManager.default.fileExists(atPath: proxyPacPath.path) {
      do {
        try FileManager.default.createDirectory(
          atPath: proxyPacPath.path, withIntermediateDirectories: true, attributes: nil)
      } catch {
        NSLog("\(error.localizedDescription)")
      }
    }
    proxyPacPath.appendPathComponent("proxy.pac")

    UserDefaults.standard.register(defaults: [
      ProxydetoxControl.autostartKey: false,
      ProxydetoxControl.portKey: 8080,
      ProxydetoxControl.pacFileKey: proxyPacPath.relativePath,
      ProxydetoxControl.negotiateKey: false,
      ProxydetoxControl.alwaysUseConnectKey: false,
      ProxydetoxControl.directFallbackKey: false,
    ])
  }

  var isRunning: Bool {
    if let proc = proxydetoxProcess {
      return proc.isRunning
    }
    return false
  }

  var isInternalNetworkAvailable: Bool {
    set { internalNetworkAvailable = newValue }
    get { return internalNetworkAvailable }
  }

  var autostart: Bool {
    get {
      return UserDefaults.standard.bool(forKey: ProxydetoxControl.autostartKey)
    }
    set {
      UserDefaults.standard.set(newValue, forKey: ProxydetoxControl.autostartKey)
    }
  }

  var port: UInt16 {
    let p = UserDefaults.standard.integer(forKey: ProxydetoxControl.portKey)
    if 1024 <= p && p < 65535 {
      return UInt16(p)
    }
    return UInt16(3128)
  }

  var pacFile: String {
    return UserDefaults.standard.string(forKey: ProxydetoxControl.pacFileKey) ?? "proxy.pac"
  }

  var negotiate: Bool {
    get {
      return UserDefaults.standard.bool(forKey: ProxydetoxControl.negotiateKey)
    }
    set {
      UserDefaults.standard.set(newValue, forKey: ProxydetoxControl.negotiateKey)
    }
  }

  var alwaysUseConnect: Bool {
    get {
      return UserDefaults.standard.bool(forKey: ProxydetoxControl.alwaysUseConnectKey)
    }
    set {
      UserDefaults.standard.set(newValue, forKey: ProxydetoxControl.alwaysUseConnectKey)
    }
  }

  var directFallback: Bool {
    get {
      return UserDefaults.standard.bool(forKey: ProxydetoxControl.directFallbackKey)
    }
    set {
      UserDefaults.standard.set(newValue, forKey: ProxydetoxControl.directFallbackKey)
    }
  }

  func start() {
    stop()
    var args = [String]()
    args.append("--port")
    args.append("\(self.port)")
    args.append("--graceful-shutdown-timeout")
    args.append("0")
    if self.negotiate {
      args.append("--negotiate")
    }
    if self.directFallback {
      args.append("--direct-fallback")
    }
    if self.alwaysUseConnect {
      args.append("--always-use-connect")
    }

    do {
      try self.proxydetoxProcess = Process.run(self.proxydetoxUrl, arguments: args) { (process) in
        print("rc: \(process.terminationStatus)")
      }
      print("pid: \(String(describing:self.proxydetoxProcess?.processIdentifier))")
    } catch {
      print("failed to run: \(error.localizedDescription)")
    }
  }

  func stop() {
    if let proc = self.proxydetoxProcess {
      proc.terminate()
      proc.waitUntilExit()
    }
  }

  func restart() {
    stop()
    start()
  }

  func setSystemProxy() {

  }
}
