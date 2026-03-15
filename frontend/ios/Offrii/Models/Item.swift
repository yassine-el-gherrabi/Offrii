import Foundation

struct Item: Codable, Identifiable, Equatable {
    let id: UUID
    let name: String
    let description: String?
    let url: String?
    let estimatedPrice: Decimal?
    let priority: Int
    let categoryId: UUID?
    let status: String
    let purchasedAt: Date?
    let createdAt: Date
    let updatedAt: Date
    let isClaimed: Bool
    let imageUrl: String?
    let links: [String]?
    let ogImageUrl: String?
    let ogTitle: String?
    let ogSiteName: String?
    let isPrivate: Bool
    let sharedCircles: [SharedCircleInfo]
    let claimedVia: String?
    let claimedName: String?

    enum CodingKeys: String, CodingKey {
        case id, name, description, url, priority, status, links
        case estimatedPrice = "estimated_price"
        case categoryId = "category_id"
        case purchasedAt = "purchased_at"
        case createdAt = "created_at"
        case updatedAt = "updated_at"
        case isClaimed = "is_claimed"
        case imageUrl = "image_url"
        case ogImageUrl = "og_image_url"
        case ogTitle = "og_title"
        case ogSiteName = "og_site_name"
        case isPrivate = "is_private"
        case sharedCircles = "shared_circles"
        case claimedVia = "claimed_via"
        case claimedName = "claimed_name"
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        id = try container.decode(UUID.self, forKey: .id)
        name = try container.decode(String.self, forKey: .name)
        description = try container.decodeIfPresent(String.self, forKey: .description)
        url = try container.decodeIfPresent(String.self, forKey: .url)
        priority = try container.decode(Int.self, forKey: .priority)
        categoryId = try container.decodeIfPresent(UUID.self, forKey: .categoryId)
        status = try container.decode(String.self, forKey: .status)
        purchasedAt = try container.decodeIfPresent(Date.self, forKey: .purchasedAt)
        createdAt = try container.decode(Date.self, forKey: .createdAt)
        updatedAt = try container.decode(Date.self, forKey: .updatedAt)
        isClaimed = try container.decode(Bool.self, forKey: .isClaimed)
        imageUrl = try container.decodeIfPresent(String.self, forKey: .imageUrl)
        links = try container.decodeIfPresent([String].self, forKey: .links)
        ogImageUrl = try container.decodeIfPresent(String.self, forKey: .ogImageUrl)
        ogTitle = try container.decodeIfPresent(String.self, forKey: .ogTitle)
        ogSiteName = try container.decodeIfPresent(String.self, forKey: .ogSiteName)
        isPrivate = (try? container.decode(Bool.self, forKey: .isPrivate)) ?? false
        sharedCircles = (try? container.decode([SharedCircleInfo].self, forKey: .sharedCircles)) ?? []
        claimedVia = try container.decodeIfPresent(String.self, forKey: .claimedVia)
        claimedName = try container.decodeIfPresent(String.self, forKey: .claimedName)

        // Backend sends estimated_price as a string ("279.00") — handle both string and number
        if let stringValue = try? container.decodeIfPresent(String.self, forKey: .estimatedPrice) {
            estimatedPrice = Decimal(string: stringValue)
        } else {
            estimatedPrice = try? container.decodeIfPresent(Decimal.self, forKey: .estimatedPrice)
        }
    }

    init(
        id: UUID, name: String, description: String?, url: String?,
        estimatedPrice: Decimal?, priority: Int, categoryId: UUID?,
        status: String, purchasedAt: Date?, createdAt: Date, updatedAt: Date,
        isClaimed: Bool, imageUrl: String?, links: [String]?,
        ogImageUrl: String?, ogTitle: String?, ogSiteName: String?,
        isPrivate: Bool, sharedCircles: [SharedCircleInfo],
        claimedVia: String?, claimedName: String?
    ) {
        self.id = id; self.name = name; self.description = description
        self.url = url; self.estimatedPrice = estimatedPrice
        self.priority = priority; self.categoryId = categoryId
        self.status = status; self.purchasedAt = purchasedAt
        self.createdAt = createdAt; self.updatedAt = updatedAt
        self.isClaimed = isClaimed; self.imageUrl = imageUrl
        self.links = links; self.ogImageUrl = ogImageUrl
        self.ogTitle = ogTitle; self.ogSiteName = ogSiteName
        self.isPrivate = isPrivate; self.sharedCircles = sharedCircles
        self.claimedVia = claimedVia; self.claimedName = claimedName
    }

    /// Display image priority: uploaded > OG > nil
    var displayImageUrl: URL? {
        if let imageUrl, let url = URL(string: imageUrl) { return url }
        if let ogImageUrl, let url = URL(string: ogImageUrl) { return url }
        return nil
    }

    var priorityLabel: String {
        switch priority {
        case 1: return String(localized: "priority.low")
        case 2: return String(localized: "priority.medium")
        case 3: return String(localized: "priority.high")
        default: return String(localized: "priority.medium")
        }
    }

    var isActive: Bool { status == "active" }
    var isPurchased: Bool { status == "purchased" }
    var isWebClaim: Bool { claimedVia == "web" }
    var isAppClaim: Bool { claimedVia == "app" }

    static func priorityLabelStatic(_ priority: Int) -> String {
        switch priority {
        case 1: return String(localized: "priority.low")
        case 2: return String(localized: "priority.medium")
        case 3: return String(localized: "priority.high")
        default: return String(localized: "priority.medium")
        }
    }

    /// Create an Item from a CircleItemResponse for display in ItemDetailSheet.
    static func fromCircleItem(_ ci: CircleItemResponse) -> Item {
        Item(
            id: ci.id,
            name: ci.name,
            description: ci.description,
            url: ci.url,
            estimatedPrice: ci.estimatedPrice,
            priority: Int(ci.priority),
            categoryId: ci.categoryId,
            status: ci.status,
            purchasedAt: nil,
            createdAt: ci.sharedAt,
            updatedAt: ci.sharedAt,
            isClaimed: ci.isClaimed,
            imageUrl: ci.imageUrl,
            links: ci.links,
            ogImageUrl: ci.ogImageUrl,
            ogTitle: ci.ogTitle,
            ogSiteName: ci.ogSiteName,
            isPrivate: false,
            sharedCircles: [],
            claimedVia: nil,
            claimedName: ci.claimedBy?.username
        )
    }
}

struct SharedCircleInfo: Codable, Identifiable, Equatable {
    let id: UUID
    let name: String
    let isDirect: Bool?

    var initial: String {
        String(name.prefix(1)).uppercased()
    }

    enum CodingKeys: String, CodingKey {
        case id, name
        case isDirect = "is_direct"
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        id = try container.decode(UUID.self, forKey: .id)
        name = try container.decode(String.self, forKey: .name)
        isDirect = try container.decodeIfPresent(Bool.self, forKey: .isDirect)
    }
}
