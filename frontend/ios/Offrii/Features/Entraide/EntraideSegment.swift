import Foundation

enum EntraideSegment: Int, CaseIterable {
    case discover = 0
    case myNeeds = 1
    case myOffers = 2

    var label: String {
        switch self {
        case .discover: return NSLocalizedString("entraide.segment.discover", comment: "")
        case .myNeeds:  return NSLocalizedString("entraide.segment.myNeeds", comment: "")
        case .myOffers: return NSLocalizedString("entraide.segment.myOffers", comment: "")
        }
    }
}
