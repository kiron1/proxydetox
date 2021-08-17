
import Foundation

public class ProxydetxoControl {
    static let autostartKey = "Autostart"
    static let portKey = "Port"
    static let pacFileKey = "PacFile"
    static let negotiateKey = "Negotiate"
    
    private var task: Process?;
    
    init() {
        UserDefaults.standard.register(defaults:[
            ProxydetxoControl.autostartKey:false,
            ProxydetxoControl.portKey:3128,
            ProxydetxoControl.negotiateKey:false,
        ])
    }
    
    var isRunning: Bool {
        get {
            if let task = task {
                return task.isRunning
            }
            return false
        }
    }
    
    public static var proxydetoxPath: String {
        get {
            let path = Bundle.main.bundlePath as NSString
            var components = path.pathComponents
            components.append("Contents")
            components.append("MacOS")
            components.append("proxydetox")
            return NSString.path(withComponents: components) as String
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
        task = Process();
        task!.executableURL = URL(fileURLWithPath: ProxydetxoControl.proxydetoxPath)
        task!.arguments = ["--port", "\(port)", "--pac-file", "\(pacFile)"]
        if negotiate {
            task!.arguments?.append("--negotiate")
        }
        do {
            try task!.run()
        } catch {
            print("Failed to run proxydetox: \(error)")
        }
    }
    
    func stop() {
        if let task = task {
            if task.isRunning {
                task.terminate()
            }
        }
    }
    
    func restart() {
        stop()
        start()
    }
    
    func setSystemProxy() {
        
    }
}
