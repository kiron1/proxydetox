//
// Utility to set system proxy configurations
//

import Foundation
import SystemConfiguration

enum ArgumentsError: Error {
  case notEnoughArguments
  case invalidPort
}

extension ArgumentsError: LocalizedError {
  public var errorDescription: String? {
    switch self {
    case .notEnoughArguments:
      return NSLocalizedString("not enough arguments provided", comment: "ArgumentsError")
    case .invalidPort:
      return NSLocalizedString("invalid port", comment: "ArgumentsError")
    }
  }
}

struct Arguments {
  var port: UInt16

  static func parse(_ args: [String]) -> Result<Arguments, Error> {
    if args.count < 2 {
      return .failure(ArgumentsError.notEnoughArguments)
    }
    guard let port = UInt16(args[1]) else {
      return .failure(ArgumentsError.invalidPort)
    }
    return .success(Arguments(port: port))
  }

  private init(port: UInt16) {
    self.port = port
  }
}

struct AuthorizationError: Error {
  var osStatus: OSStatus
}

extension AuthorizationError: LocalizedError {
  public var errorDescription: String? {
    let message = SecCopyErrorMessageString(self.osStatus, nil) as String?
    return NSLocalizedString(message ?? "unknown error", comment: "SecCopyErrorMessageString")
  }
}

class AuthorizationToken {
  var ref: AuthorizationRef

  class func acquire(flags: AuthorizationFlags) -> Result<AuthorizationToken, Error> {
    var authRef: AuthorizationRef? = nil
    let authStatus = AuthorizationCreate(nil, nil, flags, &authRef)
    if authStatus != errSecSuccess {
      return .failure(AuthorizationError(osStatus: authStatus))
    }
    return .success(AuthorizationToken(authRef!))
  }

  private init(_ ref: AuthorizationRef) {
    self.ref = ref
  }

  deinit {
    AuthorizationFree(ref, AuthorizationFlags())
  }
}

enum SysConPrefsError: Error {
  case unableToCreate
}

extension SysConPrefsError: LocalizedError {
  public var errorDescription: String? {
    switch self {
    case .unableToCreate:
      return NSLocalizedString(
        "unable to create SCPreferencesCreateWithAuthorization", comment: "Error")
    }
  }
}

class SysConPrefs {
  var prefs: SCPreferences

  class func createWithAuthorization(name: String, token: AuthorizationToken)
    -> Result<SysConPrefs, Error>
  {

    guard let prefs = SCPreferencesCreateWithAuthorization(nil, name as CFString, nil, token.ref)
    else {
      return .failure(SysConPrefsError.unableToCreate)
    }
    return .success(SysConPrefs(prefs: prefs))
  }

  private init(prefs: SCPreferences) {
    self.prefs = prefs
  }

  func getValue(key: CFString) -> CFPropertyList? {
    return SCPreferencesGetValue(prefs, key)
  }

  func setValue(key: CFString, value: CFDictionary) -> Bool {
    return SCPreferencesPathSetValue(prefs, key, value)
  }

  func commitChanges() -> Bool {
    return SCPreferencesCommitChanges(prefs)
  }

  func applyChanges() -> Bool {
    SCPreferencesApplyChanges(prefs)
  }

  func synchronize() {
    SCPreferencesSynchronize(prefs)
  }
}

/// Port of 0 means to disable
func createProxySettingsDictionary(port: UInt16) -> NSDictionary {
  let ip = "127.0.0.1"
  let enable = port != 0 ? 1 : 0

  var proxySettings: [String: AnyObject] = [:]
  proxySettings[kCFNetworkProxiesHTTPEnable as String] = enable as AnyObject
  proxySettings[kCFNetworkProxiesHTTPSEnable as String] = enable as AnyObject
  if enable != 0 {
    proxySettings[kCFNetworkProxiesHTTPProxy as String] = ip as AnyObject
    proxySettings[kCFNetworkProxiesHTTPPort as String] = port as AnyObject
    proxySettings[kCFNetworkProxiesHTTPSProxy as String] = ip as AnyObject
    proxySettings[kCFNetworkProxiesHTTPSPort as String] = port as AnyObject
  } else {
    proxySettings[kCFNetworkProxiesHTTPProxy as String] = nil
    proxySettings[kCFNetworkProxiesHTTPPort as String] = nil
    proxySettings[kCFNetworkProxiesHTTPSProxy as String] = nil
    proxySettings[kCFNetworkProxiesHTTPSPort as String] = nil
  }
  proxySettings[kCFNetworkProxiesProxyAutoDiscoveryEnable as String] = false as AnyObject
  proxySettings[kCFNetworkProxiesProxyAutoConfigEnable as String] = false as AnyObject
  proxySettings[kCFNetworkProxiesSOCKSEnable as String] = false as AnyObject
  proxySettings[kCFNetworkProxiesGopherEnable as String] = false as AnyObject
  proxySettings[kCFNetworkProxiesExceptionsList as String] =
    [
      "::1",
      "127.0.0.1",
      "localhost",
      "*.local",
    ] as AnyObject

  return proxySettings as NSDictionary
}

func run(_ args: [String]) throws {
  let arguments = try Arguments.parse(args).get()
  let authFlags: AuthorizationFlags = [.extendRights, .interactionAllowed, .preAuthorize]
  let authToken = try AuthorizationToken.acquire(flags: authFlags).get()
  let scPrefs = try SysConPrefs.createWithAuthorization(name: "Proxydetox", token: authToken).get()
  let netServices = scPrefs.getValue(key: kSCPrefNetworkServices)!

  var dirty = false
  for key in netServices.allKeys {
    let dict = netServices.object(forKey: key) as? NSDictionary
    let hardware = ((dict?[kSCEntNetInterface]) as? NSDictionary)?["Hardware"] as? String
    if hardware == "AirPort" || hardware == "Ethernet" {
      let path = "/\(kSCPrefNetworkServices)/\(key)/\(kSCEntNetProxies)"
      let proxySettings = createProxySettingsDictionary(port: arguments.port)
      let changed = (dict?[kSCEntNetProxies] as AnyObject).isNotEqual(to: proxySettings)
      if changed {
        dirty = true
        let ok = scPrefs.setValue(key: path as CFString, value: proxySettings as CFDictionary)
        if !ok {
          print("failed to set System Configuration Preferences for \(key)")
        }
      }
    }
  }
  if dirty {
    let commitOk = scPrefs.commitChanges()
    if !commitOk {
      print("failed to commit System Configuration Preferences")
    }
    let applyOk = scPrefs.applyChanges()
    if !applyOk {
      print("failed to apply System Configuration Preferences")
    }
    scPrefs.synchronize()
  }
}

@main
struct App {
  static func main() {
    do {
      try run(CommandLine.arguments)
    } catch {
      print("fatal error: \(error.localizedDescription)")
    }
  }
}
