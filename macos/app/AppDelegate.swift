import Cocoa
import ServiceManagement

extension Notification.Name {
    static let killLauncher = Notification.Name("killLauncher")
}

// @NSApplicationMain
class AppDelegate: NSObject {
    static let launcherAppId = "cc.colorto.ProxydetoxLauncher";
    let proxydetox = ProxydetxoControl();
    var statusItemController : StatusItemController?;
}


extension AppDelegate: NSApplicationDelegate {

    func applicationDidFinishLaunching(_ aNotification: Notification) {
        killLauncher();
        SMLoginItemSetEnabled(AppDelegate.launcherAppId as CFString, true)

        self.statusItemController = StatusItemController(proxydetox)
        proxydetox.start()
    }

    func applicationWillTerminate(_ notification: Notification) {
        proxydetox.stop()
    }

    func killLauncher() {
        let runningApps = NSWorkspace.shared.runningApplications
        let isRunning = !runningApps.filter { $0.bundleIdentifier == AppDelegate.launcherAppId }.isEmpty

        if isRunning {
            DistributedNotificationCenter.default().post(name: .killLauncher, object: Bundle.main.bundleIdentifier!)
        }
    }
}
