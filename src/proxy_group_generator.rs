use crate::{ProxyGroup, RegionGroupConfig, RegionTemplate, SelectGroup, UrlTestGroup, ProxyGroupCommon, get_default_region_templates};

pub struct ProxyGroupTemplateGenerator;

impl ProxyGroupTemplateGenerator {
    /// 生成地区代理组
    pub fn generate_region_groups(
        providers: &[String], 
        config: &RegionGroupConfig
    ) -> Vec<ProxyGroup> {
        let mut groups = Vec::new();
        
        if !config.enabled {
            return groups;
        }
        
        // 获取地区模板（如果配置为空，使用默认模板）
        let regions = if config.regions.is_empty() {
            get_default_region_templates()
        } else {
            config.regions.clone()
        };
        
        // 为每个地区创建代理组
        for region in &regions {
            // 创建地区选择组
            groups.push(Self::create_region_select_group(region, providers));
            
            // 如果启用了自动测试组，创建地区自动测试组
            if config.create_auto_groups {
                groups.push(Self::create_region_auto_group(region, providers));
            }
        }
        
        groups
    }
    
    /// 获取合并后的地区模板（默认 + 自定义）
    pub fn get_merged_region_templates(config: &RegionGroupConfig) -> Vec<RegionTemplate> {
        if config.regions.is_empty() {
            get_default_region_templates()
        } else {
            // 如果用户提供了自定义模板，使用用户的模板
            // 这里可以扩展为合并默认和自定义模板的逻辑
            config.regions.clone()
        }
    }
    
    /// 创建地区选择组
    pub fn create_region_select_group(
        region: &RegionTemplate, 
        providers: &[String]
    ) -> ProxyGroup {
        Self::create_region_select_group_with_global_filter(region, providers, None)
    }
    
    /// 创建地区选择组（支持全局过滤器）
    pub fn create_region_select_group_with_global_filter(
        region: &RegionTemplate, 
        providers: &[String],
        global_filter: Option<&str>
    ) -> ProxyGroup {
        let mut proxies = Vec::new();
        
        // 如果有自动测试组，添加到选择列表
        proxies.push(format!("{}-Auto", region.name));
        
        let final_filter = Self::apply_global_filter(&region.filter, global_filter);
        
        ProxyGroup::Select(SelectGroup {
            common: ProxyGroupCommon {
                name: region.name.clone(),
                proxies: Some(proxies),
                use_provider: Some(providers.to_vec()),
                url: None,
                interval: None,
                lazy: None,
                timeout: None,
                max_failed_times: None,
                disable_udp: None,
                icon: region.icon.clone(),
                filter: Some(final_filter),
            },
        })
    }
    
    /// 创建地区自动测试组
    pub fn create_region_auto_group(
        region: &RegionTemplate, 
        providers: &[String]
    ) -> ProxyGroup {
        Self::create_region_auto_group_with_global_filter(region, providers, None)
    }
    
    /// 创建地区自动测试组（支持全局过滤器）
    pub fn create_region_auto_group_with_global_filter(
        region: &RegionTemplate, 
        providers: &[String],
        global_filter: Option<&str>
    ) -> ProxyGroup {
        let final_filter = Self::apply_global_filter(&region.filter, global_filter);
        
        ProxyGroup::UrlTest(UrlTestGroup {
            common: ProxyGroupCommon {
                name: format!("{}-Auto", region.name),
                proxies: None,
                use_provider: Some(providers.to_vec()),
                url: Some("http://www.gstatic.com/generate_204".to_string()),
                interval: Some(300),
                lazy: None,
                timeout: None,
                max_failed_times: None,
                disable_udp: None,
                icon: region.icon.clone(),
                filter: Some(final_filter),
            },
            tolerance: None,
        })
    }
    
    /// 合并生成的代理组和用户自定义代理组
    pub fn merge_with_user_groups(
        generated: Vec<ProxyGroup>, 
        user: Vec<ProxyGroup>
    ) -> Vec<ProxyGroup> {
        let mut all_groups = Vec::new();
        
        // 首先添加生成的地区代理组
        all_groups.extend(generated);
        
        // 然后添加用户自定义的代理组
        all_groups.extend(user);
        
        all_groups
    }
    
    /// 应用全局过滤器到地区过滤器
    pub fn apply_global_filter(region_filter: &str, global_filter: Option<&str>) -> String {
        match global_filter {
            Some(global) => format!("({region_filter}).*{global}"),
            None => region_filter.to_string(),
        }
    }
    
    /// 验证过滤器语法是否正确
    pub fn validate_filter(filter: &str) -> Result<(), String> {
        // 这里可以添加正则表达式验证逻辑
        // 目前只做基本检查
        if filter.is_empty() {
            return Err("Filter cannot be empty".to_string());
        }
        
        // 检查是否包含基本的正则表达式结构
        if !filter.contains("(") || !filter.contains(")") {
            return Err("Filter should contain parentheses for grouping".to_string());
        }
        
        Ok(())
    }
}#[cfg
(test)]
mod tests {
    use super::*;
    use crate::{get_default_region_templates, RegionGroupConfig};

    #[test]
    fn test_generate_region_groups() {
        let providers = vec!["provider1".to_string(), "provider2".to_string()];
        let config = RegionGroupConfig {
            enabled: true,
            regions: get_default_region_templates(),
            create_auto_groups: true,
            global_filter: None,
        };

        let groups = ProxyGroupTemplateGenerator::generate_region_groups(&providers, &config);
        
        // 应该生成 6 个地区 * 2 (select + url-test) = 12 个代理组
        assert_eq!(groups.len(), 12);
        
        // 检查是否包含香港的选择组和自动测试组
        let hk_select = groups.iter().find(|g| match g {
            ProxyGroup::Select(s) => s.common.name == "HK",
            _ => false,
        });
        assert!(hk_select.is_some());
        
        let hk_auto = groups.iter().find(|g| match g {
            ProxyGroup::UrlTest(u) => u.common.name == "HK-Auto",
            _ => false,
        });
        assert!(hk_auto.is_some());
    }

    #[test]
    fn test_generate_region_groups_disabled() {
        let providers = vec!["provider1".to_string()];
        let config = RegionGroupConfig {
            enabled: false,
            regions: get_default_region_templates(),
            create_auto_groups: true,
            global_filter: None,
        };

        let groups = ProxyGroupTemplateGenerator::generate_region_groups(&providers, &config);
        assert_eq!(groups.len(), 0);
    }

    #[test]
    fn test_create_region_select_group() {
        let region = RegionTemplate {
            name: "HK".to_string(),
            display_name: Some("香港".to_string()),
            filter: "(?i)(hk|hong kong|香港|港)".to_string(),
            icon: Some("🇭🇰".to_string()),
        };
        let providers = vec!["provider1".to_string()];

        let group = ProxyGroupTemplateGenerator::create_region_select_group(&region, &providers);
        
        match group {
            ProxyGroup::Select(select) => {
                assert_eq!(select.common.name, "HK");
                assert_eq!(select.common.filter, Some("(?i)(hk|hong kong|香港|港)".to_string()));
                assert_eq!(select.common.use_provider, Some(vec!["provider1".to_string()]));
                assert_eq!(select.common.proxies, Some(vec!["HK-Auto".to_string()]));
                assert_eq!(select.common.icon, Some("🇭🇰".to_string()));
            }
            _ => panic!("Expected Select group"),
        }
    }

    #[test]
    fn test_create_region_auto_group() {
        let region = RegionTemplate {
            name: "US".to_string(),
            display_name: Some("美国".to_string()),
            filter: "(?i)(us|usa|united states|美国|美)".to_string(),
            icon: Some("🇺🇸".to_string()),
        };
        let providers = vec!["provider1".to_string(), "provider2".to_string()];

        let group = ProxyGroupTemplateGenerator::create_region_auto_group(&region, &providers);
        
        match group {
            ProxyGroup::UrlTest(url_test) => {
                assert_eq!(url_test.common.name, "US-Auto");
                assert_eq!(url_test.common.filter, Some("(?i)(us|usa|united states|美国|美)".to_string()));
                assert_eq!(url_test.common.use_provider, Some(providers));
                assert_eq!(url_test.common.url, Some("http://www.gstatic.com/generate_204".to_string()));
                assert_eq!(url_test.common.interval, Some(300));
                assert_eq!(url_test.common.icon, Some("🇺🇸".to_string()));
            }
            _ => panic!("Expected UrlTest group"),
        }
    }

    #[test]
    fn test_merge_with_user_groups() {
        let region_groups = vec![
            ProxyGroup::Select(SelectGroup {
                common: ProxyGroupCommon {
                    name: "HK".to_string(),
                    proxies: None,
                    use_provider: None,
                    url: None,
                    interval: None,
                    lazy: None,
                    timeout: None,
                    max_failed_times: None,
                    disable_udp: None,
                    icon: None,
                    filter: None,
                },
            }),
        ];
        
        let user_groups = vec![
            ProxyGroup::Select(SelectGroup {
                common: ProxyGroupCommon {
                    name: "Proxies".to_string(),
                    proxies: None,
                    use_provider: None,
                    url: None,
                    interval: None,
                    lazy: None,
                    timeout: None,
                    max_failed_times: None,
                    disable_udp: None,
                    icon: None,
                    filter: None,
                },
            }),
        ];

        let merged = ProxyGroupTemplateGenerator::merge_with_user_groups(region_groups, user_groups);
        assert_eq!(merged.len(), 2);
        
        // 地区代理组应该在前面
        match &merged[0] {
            ProxyGroup::Select(s) => assert_eq!(s.common.name, "HK"),
            _ => panic!("Expected HK group first"),
        }
        
        // 用户代理组应该在后面
        match &merged[1] {
            ProxyGroup::Select(s) => assert_eq!(s.common.name, "Proxies"),
            _ => panic!("Expected Proxies group second"),
        }
    }

    #[test]
    fn test_validate_filter() {
        // 有效的过滤器
        assert!(ProxyGroupTemplateGenerator::validate_filter("(?i)(hk|hong kong)").is_ok());
        
        // 无效的过滤器（空）
        assert!(ProxyGroupTemplateGenerator::validate_filter("").is_err());
        
        // 无效的过滤器（没有括号）
        assert!(ProxyGroupTemplateGenerator::validate_filter("hk|hong kong").is_err());
    }

    #[test]
    fn test_apply_global_filter() {
        let region_filter = "(?i)(hk|hong kong)";
        
        // 没有全局过滤器
        let result = ProxyGroupTemplateGenerator::apply_global_filter(region_filter, None);
        assert_eq!(result, region_filter);
        
        // 有全局过滤器
        let global_filter = "premium";
        let result = ProxyGroupTemplateGenerator::apply_global_filter(region_filter, Some(global_filter));
        assert_eq!(result, "((?i)(hk|hong kong)).*premium");
    }

    #[test]
    fn test_get_merged_region_templates() {
        // 使用默认模板
        let config = RegionGroupConfig {
            enabled: true,
            regions: vec![],
            create_auto_groups: true,
            global_filter: None,
        };
        
        let templates = ProxyGroupTemplateGenerator::get_merged_region_templates(&config);
        assert_eq!(templates.len(), 6); // 默认有 6 个地区
        
        // 使用自定义模板
        let custom_template = RegionTemplate {
            name: "MY".to_string(),
            display_name: Some("马来西亚".to_string()),
            filter: "(?i)(my|malaysia)".to_string(),
            icon: Some("🇲🇾".to_string()),
        };
        
        let config_with_custom = RegionGroupConfig {
            enabled: true,
            regions: vec![custom_template.clone()],
            create_auto_groups: true,
            global_filter: None,
        };
        
        let templates = ProxyGroupTemplateGenerator::get_merged_region_templates(&config_with_custom);
        assert_eq!(templates.len(), 1);
        assert_eq!(templates[0].name, "MY");
    }
}