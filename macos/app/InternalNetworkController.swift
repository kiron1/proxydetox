//
// https://www.apple.com/business/docs/site/Kerberos_Single_Sign_on_Extension_User_Guide.pdf
//

import AppKit
import Foundation

// import OSLog

class InternalNetworkController {
  // private var logger: Logger = Logger(subsystem: "cc.colorto.proxydetox", category: "InternalNetworkController")
  private var proxydetox: ProxydetoxControl

  init(_ proxydetox: ProxydetoxControl) {
    self.proxydetox = proxydetox
    DistributedNotificationCenter.default.addObserver(
      forName: Notification.Name("com.apple.KerberosPlugin.InternalNetworkAvailable"), object: nil,
      queue: nil,
      using: self.gotInternalNetworkAvailable(notification:)
    )

    DistributedNotificationCenter.default.addObserver(
      forName: Notification.Name("com.apple.KerberosPlugin.InternalNetworkNotAvailable"),
      object: nil,
      queue: nil,
      using: self.gotInternalNetworkNotAvailable(notification:)
    )
  }

  func gotInternalNetworkAvailable(notification: Notification) {
    // self.logger.info("internal network available")
    proxydetox.isInternalNetworkAvailable = true
  }

  func gotInternalNetworkNotAvailable(notification: Notification) {
    // self.logger.info("internal network not available")
    proxydetox.isInternalNetworkAvailable = false

  }
}
