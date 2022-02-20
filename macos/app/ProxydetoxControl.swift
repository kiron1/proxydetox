
import Foundation
import Proxydetox

public class ProxydetxoControl {
    static let autostartKey = "Autostart"
    static let portKey = "Port"
    static let pacFileKey = "PacFile"
    static let negotiateKey = "Negotiate"

    private var pd: OpaquePointer? = nil;
    private var worker: Thread? = nil

    init() {
        var proxyPacPath = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first!
        proxyPacPath.appendPathComponent("Proxydetox")

        if !FileManager.default.fileExists(atPath: proxyPacPath.path) {
            do {
                try FileManager.default.createDirectory(atPath: proxyPacPath.path, withIntermediateDirectories: true, attributes: nil)
            } catch {
                NSLog("\(error.localizedDescription)")
            }
        }
        proxyPacPath.appendPathComponent("proxy.pac")

        UserDefaults.standard.register(defaults:[
            ProxydetxoControl.autostartKey:false,
            ProxydetxoControl.portKey:3128,
            ProxydetxoControl.pacFileKey:proxyPacPath.relativePath,
            ProxydetxoControl.negotiateKey:false,
        ])
    }

    var isRunning: Bool {
        get {
            return pd != nil
        }
    }

    var autostart: Bool {
        get {
            return UserDefaults.standard.bool(forKey: ProxydetxoControl.autostartKey)
        }
        set {
            UserDefaults.standard.set(newValue, forKey: ProxydetxoControl.autostartKey)
        }
    }

    var port: UInt16 {
        get {
            let p = UserDefaults.standard.integer(forKey: ProxydetxoControl.portKey);
            if 1024 <= p && p < 65535 {
                return UInt16(p)
            }
            return UInt16(3128)
        }
    }

    var pacFile: String {
        get {
            return UserDefaults.standard.string(forKey: ProxydetxoControl.pacFileKey) ?? "proxy.pac"
        }
    }

    var negotiate: Bool {
        get {
            return UserDefaults.standard.bool(forKey: ProxydetxoControl.negotiateKey)
        }
        set {
            UserDefaults.standard.set(newValue, forKey: ProxydetxoControl.negotiateKey)
        }
    }

    func start() {
        stop()
        pd = pacFile.withCString { (filePath) -> OpaquePointer? in
            proxydetox_new(filePath, negotiate)
        }
        worker = Thread(
            target:self,
            selector:#selector(ProxydetxoControl.run),
            object:nil)
        if let worker = worker {
            worker.start()
        }
    }

    func stop() {
        if let pd = pd {
            // Send the signal to shutdown
            proxydetox_shutdown(pd)
        }
        pd = nil
    }

    func restart() {
        stop()
        start()
    }

    @objc func run() {
        // Keep a copy of the ProxydetoxServer pointer,
        // since `stop` will send the signal to shutdown
        // and after the shutdown is finished we can drop
        // the ProxydetoxServer.
        let thisPd = pd
        proxydetox_run(thisPd, port)
        proxydetox_drop(thisPd)
    }

    func setSystemProxy() {

    }
}
