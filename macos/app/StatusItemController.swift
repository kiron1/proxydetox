import Cocoa
import ServiceManagement

class StatusItemController: NSObject, NSApplicationDelegate, NSMenuDelegate {
  var statusBarItem: NSStatusItem!
  var proxydetox: ProxydetoxControl

  init(_ proxydetox: ProxydetoxControl) {
    self.proxydetox = proxydetox

    statusBarItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.squareLength)
    statusBarItem.button?.title = "𝌓"
    statusBarItem.menu = NSMenu()

    super.init()

    statusBarItem.menu!.delegate = self
  }

  func menuNeedsUpdate(_ menu: NSMenu) {
    menu.removeAllItems()

    //        let autostartItem = menu.addItem(withTitle: "Start Proxydetox on Logon",
    //                                         action: #selector(StatusItemController.autostart),
    //                                         keyEquivalent: "")
    //        autostartItem.state = proxydetox.autostart ? .on : .off
    //        autostartItem.target = self
    //
    //        menu.addItem(NSMenuItem.separator())

    if proxydetox.isRunning {
      menu.addItem(
        withTitle: "Stop",
        action: #selector(StatusItemController.stop),
        keyEquivalent: ""
      ).target = self
      menu.addItem(NSMenuItem.separator())
      menu.addItem(
        withTitle: "Port: \(proxydetox.port)",
        action: nil,
        keyEquivalent: "")

      menu.addItem(
        withTitle: "PAC: \(proxydetox.pacFile)",
        action: nil,
        keyEquivalent: "")
    } else {
      menu.addItem(
        withTitle: "Start",
        action: #selector(StatusItemController.start),
        keyEquivalent: ""
      ).target = self
    }
    menu.addItem(NSMenuItem.separator())

    let alwaysUseConnectItem = menu.addItem(
      withTitle: "always use CONNECT",
      action: #selector(StatusItemController.alwaysUseConnect),
      keyEquivalent: "")
    alwaysUseConnectItem.state = proxydetox.alwaysUseConnect ? .on : .off
    alwaysUseConnectItem.target = self

    let directFallbackItem = menu.addItem(
      withTitle: "DIRECT fallback",
      action: #selector(StatusItemController.directFallback),
      keyEquivalent: "")
    directFallbackItem.state = proxydetox.directFallback ? .on : .off
    directFallbackItem.target = self

    let negotiateItem = menu.addItem(
      withTitle: "Negotiate",
      action: #selector(StatusItemController.negotiate),
      keyEquivalent: "")
    negotiateItem.state = proxydetox.negotiate ? .on : .off
    negotiateItem.target = self
    if proxydetox.negotiate {
      let networkAvailable = menu.addItem(
        withTitle: "Internal network available",
        action: nil,
        keyEquivalent: "")
      networkAvailable.state = proxydetox.isInternalNetworkAvailable ? .on : .off
    }

    menu.addItem(NSMenuItem.separator())

    menu.addItem(
      withTitle: "Quit Proxydetox",
      action: #selector(StatusItemController.quit),
      keyEquivalent: "q"
    ).target = self

  }

  @objc func autostart(sender: NSStatusBarButton) {
  }

  @objc func start(sender: NSStatusBarButton) {
    //let event = NSApp.currentEvent!
    proxydetox.start()
  }

  @objc func stop(sender: NSStatusBarButton) {
    //let event = NSApp.currentEvent!
    proxydetox.stop()
  }

  @objc func alwaysUseConnect(sender: NSStatusBarButton) {
    proxydetox.alwaysUseConnect = !proxydetox.alwaysUseConnect
    proxydetox.restart()
  }

  @objc func directFallback(sender: NSStatusBarButton) {
    proxydetox.directFallback = !proxydetox.directFallback
    proxydetox.restart()
  }

  @objc func negotiate(sender: NSStatusBarButton) {
    proxydetox.negotiate = !proxydetox.negotiate
    proxydetox.restart()
  }

  @objc func quit(sender: NSStatusBarButton) {
    //let event = NSApp.currentEvent!
    NSApp.terminate(sender)
  }
}
